//! Representação e validação com tipagem forte dos registros da tabela Simples / MEI.

use crate::converter::{ConvertErr, TableDescriptor};
use crate::util::{ErroCampo, clean_quotes, opt_str, parse_opt_u32, split_csv_line};
use arrow::array::{ArrayRef, StringBuilder, UInt32Builder};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

/// Registro validado e fortemente tipado da tabela Simples / MEI da Receita Federal.
#[derive(Debug, PartialEq)]
pub struct Simples<'a> {
    pub cnpj_basico: &'a [u8],
    pub opcao_simples: Option<&'a str>,
    pub data_opcao_simples: Option<u32>,
    pub data_exclusao_simples: Option<u32>,
    pub opcao_mei: Option<&'a str>,
    pub data_opcao_mei: Option<u32>,
    pub data_exclusao_mei: Option<u32>,
}

impl<'a> Simples<'a> {
    /// Faz o parse de uma fatia de bytes de uma linha CSV da tabela Simples.
    ///
    /// ### Parâmetros
    /// - `line`: Fatia de bytes referente a uma linha única de texto CSV.
    ///
    /// ### Retorno
    /// - `Result<Self, ErroCampo>`: Estrutura tipada ou erro de parsing/validação.
    pub fn parse_line(line: &'a [u8]) -> Result<Self, ErroCampo> {
        let campos: [&'a [u8]; 7] = split_csv_line::<7>(line)?;

        let cnpj_basico = clean_quotes(campos[0]);
        if cnpj_basico.len() != 8 || !cnpj_basico.iter().all(|b| b.is_ascii_alphanumeric()) {
            return Err(ErroCampo::CnpjInvalido);
        }

        let opcao_simples = opt_str(clean_quotes(campos[1]))?;
        let data_opcao_simples = parse_opt_u32(clean_quotes(campos[2]))?;
        let data_exclusao_simples = parse_opt_u32(clean_quotes(campos[3]))?;
        let opcao_mei = opt_str(clean_quotes(campos[4]))?;
        let data_opcao_mei = parse_opt_u32(clean_quotes(campos[5]))?;
        let data_exclusao_mei = parse_opt_u32(clean_quotes(campos[6]))?;

        Ok(Self {
            cnpj_basico,
            opcao_simples,
            data_opcao_simples,
            data_exclusao_simples,
            opcao_mei,
            data_opcao_mei,
            data_exclusao_mei,
        })
    }
}

/// Descritor da tabela Simples / MEI responsável por seu esquema Arrow e montagem de lotes.
#[derive(Debug, Clone, Copy, Default)]
pub struct SimplesDescriptor;

impl TableDescriptor for SimplesDescriptor {
    fn name(&self) -> &'static str {
        "simples"
    }

    fn schema(&self) -> SchemaRef {
        Arc::new(Schema::new(vec![
            Field::new("cnpj_basico", DataType::Utf8, false),
            Field::new("opcao_simples", DataType::Utf8, true),
            Field::new("data_opcao_simples", DataType::UInt32, true),
            Field::new("data_exclusao_simples", DataType::UInt32, true),
            Field::new("opcao_mei", DataType::Utf8, true),
            Field::new("data_opcao_mei", DataType::UInt32, true),
            Field::new("data_exclusao_mei", DataType::UInt32, true),
        ]))
    }

    fn build_batch(&self, slice: &[u8]) -> Result<Option<RecordBatch>, ConvertErr> {
        if slice.is_empty() {
            return Ok(None);
        }

        let mut cnpj_b = StringBuilder::new();
        let mut opc_simples_b = StringBuilder::new();
        let mut dt_opc_simples_b = UInt32Builder::new();
        let mut dt_exc_simples_b = UInt32Builder::new();
        let mut opc_mei_b = StringBuilder::new();
        let mut dt_opc_mei_b = UInt32Builder::new();
        let mut dt_exc_mei_b = UInt32Builder::new();

        let mut row_count: usize = 0;
        let mut start: usize = 0;

        for (i, &byte) in slice.iter().enumerate() {
            if byte == b'\n' {
                let line: &[u8] = &slice[start..i];
                start = i + 1;
                let line = line.strip_suffix(b"\r").unwrap_or(line);
                if line.is_empty() {
                    continue;
                }

                let Ok(simples) = Simples::parse_line(line) else {
                    continue;
                };
                let Ok(cnpj_str) = std::str::from_utf8(simples.cnpj_basico) else {
                    continue;
                };

                cnpj_b.append_value(cnpj_str);
                append_opt_str(&mut opc_simples_b, simples.opcao_simples);
                append_opt_u32(&mut dt_opc_simples_b, simples.data_opcao_simples);
                append_opt_u32(&mut dt_exc_simples_b, simples.data_exclusao_simples);
                append_opt_str(&mut opc_mei_b, simples.opcao_mei);
                append_opt_u32(&mut dt_opc_mei_b, simples.data_opcao_mei);
                append_opt_u32(&mut dt_exc_mei_b, simples.data_exclusao_mei);

                row_count += 1;
            }
        }

        if row_count == 0 {
            return Ok(None);
        }

        let columns: Vec<ArrayRef> = vec![
            Arc::new(cnpj_b.finish()),
            Arc::new(opc_simples_b.finish()),
            Arc::new(dt_opc_simples_b.finish()),
            Arc::new(dt_exc_simples_b.finish()),
            Arc::new(opc_mei_b.finish()),
            Arc::new(dt_opc_mei_b.finish()),
            Arc::new(dt_exc_mei_b.finish()),
        ];

        Ok(Some(RecordBatch::try_new(self.schema(), columns)?))
    }
}

#[inline]
fn append_opt_str(b: &mut StringBuilder, val: Option<&str>) {
    match val {
        Some(v) => b.append_value(v),
        None => b.append_null(),
    }
}

#[inline]
fn append_opt_u32(b: &mut UInt32Builder, val: Option<u32>) {
    match val {
        Some(v) => b.append_value(v),
        None => b.append_null(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_simples_line() {
        let line = b"\"12345678\";\"S\";\"20200101\";\"20211231\";\"N\";\"\";\"\"";
        let reg = Simples::parse_line(line).expect("Valid simples line");
        assert_eq!(reg.cnpj_basico, b"12345678");
        assert_eq!(reg.opcao_simples, Some("S"));
        assert_eq!(reg.data_opcao_simples, Some(20200101));
        assert_eq!(reg.data_exclusao_simples, Some(20211231));
        assert_eq!(reg.opcao_mei, Some("N"));
        assert_eq!(reg.data_opcao_mei, None);
        assert_eq!(reg.data_exclusao_mei, None);
    }

    #[test]
    fn test_simples_insufficient_fields() {
        let line = b"\"12345678\";\"S\"";
        assert_eq!(
            Simples::parse_line(line),
            Err(ErroCampo::CamposInsuficientes)
        );
    }

    #[test]
    fn test_simples_invalid_cnpj() {
        let line = b"\"1234\";\"S\";\"\";\"\";\"\";\"\";\"\"";
        assert_eq!(Simples::parse_line(line), Err(ErroCampo::CnpjInvalido));
    }

    #[test]
    fn test_simples_descriptor_schema_e_batch() {
        let descriptor = SimplesDescriptor;
        let schema = descriptor.schema();
        assert_eq!(schema.fields().len(), 7);
        assert_eq!(schema.field(0).name(), "cnpj_basico");
        assert_eq!(schema.field(4).name(), "opcao_mei");

        let csv_data = b"\"12345678\";\"S\";\"20200101\";\"20211231\";\"N\";\"\";\"\"\n";
        let batch = descriptor
            .build_batch(csv_data)
            .expect("Sucesso ao construir batch de Simples")
            .expect("RecordBatch não nulo");

        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 7);
    }
}
