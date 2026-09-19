//! Conversor colunar de alta vazão de registros de Empresas para Apache Parquet.

use crate::empresa::Empresa;
use crate::reader::MmapReader;
use arrow::array::{ArrayRef, Float64Builder, StringBuilder, UInt8Builder, UInt16Builder};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use rayon::prelude::*;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

/// Erros tipados durante a leitura, conversão colunar e gravação do Parquet.
#[derive(Debug, thiserror::Error)]
pub enum ConvertErr {
    #[error("Erro de I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("Erro no Arrow: {0}")]
    Arrow(#[from] arrow::error::ArrowError),
    #[error("Erro no Parquet: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),
}

/// Retorna a definição do esquema colunar do Apache Arrow para a tabela de Empresas.
#[inline]
pub fn empresa_schema() -> SchemaRef {
    let fields: Vec<Field> = vec![
        Field::new("cnpj_basico", DataType::Utf8, false),
        Field::new("razao_social", DataType::Utf8, false),
        Field::new("natureza_juridica", DataType::UInt16, false),
        Field::new("qualificacao_responsavel", DataType::UInt16, false),
        Field::new("capital_social", DataType::Float64, false),
        Field::new("porte", DataType::UInt8, false),
        Field::new("ente_federativo", DataType::Utf8, true),
    ];
    Arc::new(Schema::new(fields))
}

/// Constrói um `RecordBatch` a partir de uma fatia contígua de bytes contendo linhas CSV.
pub fn build_batch(slice: &[u8], schema: &SchemaRef) -> Result<Option<RecordBatch>, ConvertErr> {
    if slice.is_empty() {
        return Ok(None);
    }

    let mut cnpj_b: StringBuilder = StringBuilder::new();
    let mut razao_b: StringBuilder = StringBuilder::new();
    let mut nat_jur_b: UInt16Builder = UInt16Builder::new();
    let mut qualif_b: UInt16Builder = UInt16Builder::new();
    let mut capital_b: Float64Builder = Float64Builder::new();
    let mut porte_b: UInt8Builder = UInt8Builder::new();
    let mut ente_fed_b: StringBuilder = StringBuilder::new();

    let mut count: usize = 0;
    let mut start: usize = 0;

    for (i, &byte) in slice.iter().enumerate() {
        if byte == b'\n' {
            let line: &[u8] = &slice[start..i];
            start = i + 1;

            // Ignora linhas vazias
            if line.is_empty() || (line.len() == 1 && line[0] == b'\r') {
                continue;
            }

            let Ok(empresa) = Empresa::parse_line(line) else {
                continue;
            };
            let Ok(cnpj_str) = std::str::from_utf8(empresa.cnpj) else {
                continue;
            };

            cnpj_b.append_value(cnpj_str);
            razao_b.append_value(empresa.razao);
            nat_jur_b.append_value(empresa.nat_jur);
            qualif_b.append_value(empresa.qualif);
            capital_b.append_value(empresa.capital);
            porte_b.append_value(empresa.porte);

            match empresa.ente_fed {
                Some(ente) => ente_fed_b.append_value(ente),
                None => ente_fed_b.append_null(),
            }

            count += 1;
        }
    }

    if count == 0 {
        return Ok(None);
    }

    let columns: Vec<ArrayRef> = vec![
        Arc::new(cnpj_b.finish()),
        Arc::new(razao_b.finish()),
        Arc::new(nat_jur_b.finish()),
        Arc::new(qualif_b.finish()),
        Arc::new(capital_b.finish()),
        Arc::new(porte_b.finish()),
        Arc::new(ente_fed_b.finish()),
    ];

    let batch: RecordBatch = RecordBatch::try_new(Arc::clone(schema), columns)?;
    Ok(Some(batch))
}

/// Converte um arquivo CSV de Empresas para Parquet em alta vazão.
///
/// Mapeia o arquivo com `memmap2`, fatia em blocos e gera o Parquet comprimido com Snappy.
pub fn to_parquet<P1: AsRef<Path>, P2: AsRef<Path>>(src: P1, dst: P2) -> Result<usize, ConvertErr> {
    let reader: MmapReader = MmapReader::from_path(src)?;
    if reader.is_empty() {
        return Ok(0);
    }

    let num_threads: usize = rayon::current_num_threads().max(1);
    let boundaries: Vec<(usize, usize)> = reader.chunk_boundaries(num_threads);
    let bytes: &[u8] = reader.as_bytes();
    let schema: SchemaRef = empresa_schema();

    // Processamento paralelo de cada chunk para gerar os lotes RecordBatch
    let batches: Vec<RecordBatch> = boundaries
        .par_iter()
        .filter_map(|&(start, end)| {
            let chunk_slice: &[u8] = &bytes[start..end];
            build_batch(chunk_slice, &schema).ok().flatten()
        })
        .collect();

    let total_rows: usize = batches.iter().map(|b: &RecordBatch| b.num_rows()).sum();
    if total_rows == 0 {
        return Ok(0);
    }

    let file: File = File::create(dst)?;
    let props: WriterProperties = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .build();

    let mut writer: ArrowWriter<File> = ArrowWriter::try_new(file, schema, Some(props))?;
    for batch in &batches {
        writer.write(batch)?;
    }
    writer.close()?;

    Ok(total_rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_empresa_schema_fields() {
        let schema: SchemaRef = empresa_schema();
        assert_eq!(schema.fields().len(), 7);
        assert_eq!(schema.field(0).name(), "cnpj_basico");
        assert_eq!(schema.field(1).name(), "razao_social");
        assert_eq!(schema.field(4).name(), "capital_social");
    }

    #[test]
    fn test_build_batch_from_slice() {
        let csv_data: &[u8] = b"\"12345678\";\"EMPRESA A\";\"2062\";\"49\";\"1000,00\";\"01\";\"\"\n\"87654321\";\"EMPRESA B\";\"2062\";\"49\";\"5000,50\";\"03\";\"BRASILIA\"\n";
        let schema: SchemaRef = empresa_schema();
        let batch_opt: Option<RecordBatch> =
            build_batch(csv_data, &schema).expect("Valid batch creation");

        let batch: RecordBatch = batch_opt.expect("Non-empty batch");
        assert_eq!(batch.num_rows(), 2);
        assert_eq!(batch.num_columns(), 7);
    }

    #[test]
    fn test_to_parquet_file() {
        let temp_dir: std::path::PathBuf = std::env::temp_dir();
        let csv_path: std::path::PathBuf = temp_dir.join("test_empresas.csv");
        let parquet_path: std::path::PathBuf = temp_dir.join("test_empresas.parquet");

        let mut file: File = File::create(&csv_path).expect("Create temp csv");
        file.write_all(
            b"\"12345678\";\"EMPRESA ALFA\";\"2062\";\"49\";\"15000,00\";\"03\";\"\"\n\
              \"87654321\";\"EMPRESA BETA\";\"2062\";\"49\";\"25000,50\";\"05\";\"SAO PAULO\"\n",
        )
        .expect("Write to temp csv");
        drop(file);

        let rows: usize =
            to_parquet(&csv_path, &parquet_path).expect("Conversion to parquet succeeded");
        assert_eq!(rows, 2);
        assert!(parquet_path.exists());

        // Limpeza dos arquivos temporários
        let _ = std::fs::remove_file(csv_path);
        let _ = std::fs::remove_file(parquet_path);
    }
}
