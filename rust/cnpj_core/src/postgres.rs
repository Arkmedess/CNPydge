//! PostgreSQL writer using COPY (text format, Phase 1).
//!
//! Writes batches to Postgres using COPY STDIN for high throughput.
//! Phase 1 uses text COPY for simplicity; Phase 2 will upgrade to COPY BINARY.
//!
//! Indices are dropped before import and recreated after (per ADR-0007).

use crate::channels::Batch;
use crate::metrics::PipelineMetrics;
use postgres::{Client, NoTls};
use std::sync::Arc;

/// Table name for CNPJ empresas data.
const TABLE_NAME: &str = "empresas";

/// Column order matching the COPY protocol.
const COLUMNS: &[&str] = &[
    "cnpj",
    "razao_social",
    "cnae_fiscal",
    "situacao_cadastral",
    "data_situacao_cadastral",
    "uf",
    "codigo_municipio",
    "cep",
    "ddd_telefone_1",
    "ddd_telefone_2",
    "ddd_fax",
    "data_abertura",
    "natureza_juridica",
    "qualificacao_representante_legal",
    "porte_empresa",
    "opcao_simples_nacional",
    "data_opcao_simples_nacional",
    "data_exclusao_simples_nacional",
    "opcao_mei",
    "situacao_especial",
    "data_situacao_especial",
    "capital_social",
];

/// SQL to create the empresas table if it doesn't exist.
pub const CREATE_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS empresas (
    cnpj VARCHAR(14) PRIMARY KEY,
    razao_social TEXT,
    cnae_fiscal VARCHAR(7),
    situacao_cadastral VARCHAR(2),
    data_situacao_cadastral VARCHAR(8),
    uf VARCHAR(2),
    codigo_municipio VARCHAR(7),
    cep VARCHAR(8),
    ddd_telefone_1 VARCHAR(2),
    ddd_telefone_2 VARCHAR(2),
    ddd_fax VARCHAR(2),
    data_abertura VARCHAR(8),
    natureza_juridica VARCHAR(4),
    qualificacao_representante_legal VARCHAR(2),
    porte_empresa VARCHAR(2),
    opcao_simples_nacional VARCHAR(1),
    data_opcao_simples_nacional VARCHAR(8),
    data_exclusao_simples_nacional VARCHAR(8),
    opcao_mei VARCHAR(1),
    situacao_especial VARCHAR(2),
    data_situacao_especial VARCHAR(8),
    capital_social VARCHAR(15)
);
"#;

/// SQL to recreate indices after import.
pub fn create_indices_sql() -> Vec<String> {
    vec![
        format!("CREATE INDEX IF NOT EXISTS idx_empresas_uf ON {} (uf);", TABLE_NAME),
        format!("CREATE INDEX IF NOT EXISTS idx_empresas_cnae ON {} (cnae_fiscal);", TABLE_NAME),
        format!("CREATE INDEX IF NOT EXISTS idx_empresas_situacao ON {} (situacao_cadastral);", TABLE_NAME),
    ]
}

/// Errors that can occur during writing.
#[derive(Debug)]
pub enum WriteError {
    Postgres(postgres::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteError::Postgres(e) => write!(f, "Postgres error: {}", e),
            WriteError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for WriteError {}

impl From<postgres::Error> for WriteError {
    fn from(e: postgres::Error) -> Self {
        WriteError::Postgres(e)
    }
}

impl From<std::io::Error> for WriteError {
    fn from(e: std::io::Error) -> Self {
        WriteError::Io(e)
    }
}

/// Writer that sends batches to Postgres via COPY.
pub struct PostgresWriter {
    client: Client,
    metrics: Arc<PipelineMetrics>,
}

impl PostgresWriter {
    /// Connect to Postgres and prepare the writer.
    pub fn connect(dsn: &str, metrics: Arc<PipelineMetrics>) -> Result<Self, postgres::Error> {
        let client = Client::connect(dsn, NoTls)?;
        Ok(Self { client, metrics })
    }

    /// Initialize the database: create table.
    pub fn initialize(&mut self) -> Result<(), postgres::Error> {
        self.client.execute(CREATE_TABLE_SQL, &[])?;
        Ok(())
    }

    /// Finalize: recreate indices, run VACUUM ANALYZE.
    pub fn finalize(&mut self) -> Result<(), postgres::Error> {
        for sql in create_indices_sql() {
            self.client.execute(&sql, &[])?;
        }
        self.client.execute("VACUUM ANALYZE empresas", &[])?;
        Ok(())
    }

    /// Write a single batch to Postgres using COPY (text format).
    pub fn write_batch(&mut self, batch: &Batch) -> Result<usize, WriteError> {
        if batch.is_empty() {
            return Ok(0);
        }

        let columns_str = COLUMNS.join(", ");
        let copy_sql = format!("COPY {} ({}) FROM STDIN", TABLE_NAME, columns_str);

        let mut sink = self.client.copy_in(&copy_sql)?;

        for record in batch {
            let line = build_copy_line(record);
            // CopyInWriter implements std::io::Write
            std::io::Write::write_all(&mut sink, line.as_bytes())?;
            std::io::Write::write_all(&mut sink, b"\n")?;
        }

        sink.finish()?;

        let count = batch.len();
        self.metrics
            .records_written
            .fetch_add(count as u64, std::sync::atomic::Ordering::Relaxed);

        Ok(count)
    }
}

/// Build a tab-separated line for COPY text format.
/// Empty fields are written as `\N` (Postgres NULL marker).
fn build_copy_line(record: &crate::channels::NormalizedRecord) -> String {
    let fields: Vec<&[u8]> = vec![
        &record.cnpj,
        &record.razao_social,
        &record.cnae_fiscal,
        &record.situacao_cadastral,
        &record.data_situacao_cadastral,
        &record.uf,
        &record.codigo_municipio,
        &record.cep,
        &record.ddd_telefone_1,
        &record.ddd_telefone_2,
        &record.ddd_fax,
        &record.data_abertura,
        &record.natureza_juridica,
        &record.qualificacao_representante_legal,
        &record.porte_empresa,
        &record.opcao_simples_nacional,
        &record.data_opcao_simples_nacional,
        &record.data_exclusao_simples_nacional,
        &record.opcao_mei,
        &record.situacao_especial,
        &record.data_situacao_especial,
        &record.capital_social,
    ];

    fields
        .iter()
        .map(|f| {
            if f.is_empty() {
                "\\N".to_string()
            } else {
                String::from_utf8_lossy(f).into_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\t")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channels::NormalizedRecord;

    fn empty_record() -> NormalizedRecord {
        NormalizedRecord {
            cnpj: *b"12345678000195",
            razao_social: b"Empresa Teste".to_vec(),
            cnae_fiscal: b"6201501".to_vec(),
            situacao_cadastral: b"02".to_vec(),
            data_situacao_cadastral: b"20200101".to_vec(),
            uf: b"SP".to_vec(),
            codigo_municipio: b"3550308".to_vec(),
            cep: b"01310100".to_vec(),
            ddd_telefone_1: b"11".to_vec(),
            ddd_telefone_2: Vec::new(),
            ddd_fax: Vec::new(),
            data_abertura: b"20200101".to_vec(),
            natureza_juridica: b"2062".to_vec(),
            qualificacao_representante_legal: b"49".to_vec(),
            porte_empresa: Vec::new(),
            opcao_simples_nacional: b"N".to_vec(),
            data_opcao_simples_nacional: Vec::new(),
            data_exclusao_simples_nacional: Vec::new(),
            opcao_mei: b"N".to_vec(),
            situacao_especial: Vec::new(),
            data_situacao_especial: Vec::new(),
            capital_social: b"100000.00".to_vec(),
        }
    }

    #[test]
    fn test_build_copy_line() {
        let record = empty_record();
        let line = build_copy_line(&record);
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 22);
        assert_eq!(fields[0], "12345678000195");
        assert_eq!(fields[1], "Empresa Teste");
        assert_eq!(fields[5], "SP");
        // Empty fields should be \N
        assert_eq!(fields[9], "\\N"); // ddd_telefone_2
        assert_eq!(fields[10], "\\N"); // ddd_fax
    }
}
