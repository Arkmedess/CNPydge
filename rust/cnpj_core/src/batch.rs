//! Batch accumulator.
//!
//! Collects normalized records into batches of configurable size.
//! The batch Vec is recycled (not reallocated) between sends.
//!
//! Default batch size: 100,000 records.

use crate::channels::{Batch, NormalizedRecord};

/// Batch builder that accumulates records and flushes when full.
pub struct BatchBuilder {
    buffer: Batch,
    capacity: usize,
}

impl BatchBuilder {
    /// Create a new batch builder with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Push a record into the current batch.
    /// Returns `Some(batch)` if the batch is full and ready to send.
    #[inline]
    pub fn push(&mut self, record: NormalizedRecord) -> Option<Batch> {
        self.buffer.push(record);
        if self.buffer.len() >= self.capacity {
            Some(self.flush())
        } else {
            None
        }
    }

    /// Flush the current batch, returning all accumulated records.
    /// The internal buffer is reused (cleared, not deallocated).
    pub fn flush(&mut self) -> Batch {
        std::mem::take(&mut self.buffer)
    }

    /// Check if the batch has any records.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Number of records currently in the batch.
    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Batch capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(cnpj_suffix: u8) -> NormalizedRecord {
        let mut cnpj = [0u8; 14];
        cnpj[13] = cnpj_suffix;
        NormalizedRecord {
            cnpj,
            razao_social: Vec::new(),
            cnae_fiscal: Vec::new(),
            situacao_cadastral: Vec::new(),
            data_situacao_cadastral: Vec::new(),
            uf: Vec::new(),
            codigo_municipio: Vec::new(),
            cep: Vec::new(),
            ddd_telefone_1: Vec::new(),
            ddd_telefone_2: Vec::new(),
            ddd_fax: Vec::new(),
            data_abertura: Vec::new(),
            natureza_juridica: Vec::new(),
            qualificacao_representante_legal: Vec::new(),
            porte_empresa: Vec::new(),
            opcao_simples_nacional: Vec::new(),
            data_opcao_simples_nacional: Vec::new(),
            data_exclusao_simples_nacional: Vec::new(),
            opcao_mei: Vec::new(),
            situacao_especial: Vec::new(),
            data_situacao_especial: Vec::new(),
            capital_social: Vec::new(),
        }
    }

    #[test]
    fn test_batch_not_full() {
        let mut builder = BatchBuilder::new(10);
        assert!(builder.push(make_record(1)).is_none());
        assert!(builder.push(make_record(2)).is_none());
        assert_eq!(builder.len(), 2);
    }

    #[test]
    fn test_batch_flush_on_full() {
        let mut builder = BatchBuilder::new(3);
        assert!(builder.push(make_record(1)).is_none());
        assert!(builder.push(make_record(2)).is_none());
        let batch = builder.push(make_record(3)).unwrap();
        assert_eq!(batch.len(), 3);
        assert_eq!(batch[0].cnpj[13], 1);
        assert_eq!(batch[2].cnpj[13], 3);
    }

    #[test]
    fn test_batch_recycle() {
        let mut builder = BatchBuilder::new(2);
        let _ = builder.push(make_record(1));
        let batch1 = builder.push(make_record(2)).unwrap();
        drop(batch1);

        // Buffer should be reused
        let _ = builder.push(make_record(3));
        let batch2 = builder.push(make_record(4)).unwrap();
        assert_eq!(batch2.len(), 2);
        assert_eq!(batch2[0].cnpj[13], 3);
    }
}
