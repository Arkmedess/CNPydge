//! Conversor colunar unificado de alta vazão para Apache Parquet.

use crate::reader::MmapReader;
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use rayon::prelude::*;
use std::fs::File;
use std::path::Path;

/// Erros tipados durante a leitura, conversão colunar e gravação do Parquet.
#[derive(Debug, thiserror::Error)]
pub enum ConvertErr {
    /// Falha de I/O na leitura do arquivo de origem ou criação do arquivo de destino.
    #[error("Erro de I/O: {0}")]
    Io(#[from] std::io::Error),
    /// Falha durante a construção de arrays ou batches no Apache Arrow.
    #[error("Erro no Arrow: {0}")]
    Arrow(#[from] arrow::error::ArrowError),
    /// Falha na serialização ou escrita do arquivo Apache Parquet.
    #[error("Erro no Parquet: {0}")]
    Parquet(#[from] parquet::errors::ParquetError),
    /// Identificador de entidade não suportado pelo conversor.
    #[error("Tabela não suportada: {0}")]
    UnsupportedTable(String),
}

/// Contrato declarativo para definição de esquemas e montagem colunar de tabelas da RFB.
pub trait TableDescriptor: Send + Sync {
    /// Identificador textual canônico da entidade (ex.: "empresas").
    fn name(&self) -> &'static str;

    /// Retorna a referência imutável ao esquema colunar Apache Arrow correspondente.
    fn schema(&self) -> SchemaRef;

    /// Converte uma fatia contígua de bytes CSV em um RecordBatch Arrow em memória.
    fn build_batch(&self, slice: &[u8]) -> Result<Option<RecordBatch>, ConvertErr>;
}

/// Identificador fortemente tipado das tabelas do CNPJ suportadas.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableKind {
    /// Tabela principal de dados cadastrais de Empresas.
    Empresas,
    /// Tabela do Quadro de Sócios e Administradores (QSA).
    Socios,
    /// Tabela de unidades e dados de Estabelecimentos (matriz/filial).
    Estabelecimentos,
    /// Tabela de histórico e opção pelo Simples Nacional e MEI.
    Simples,
    /// Tabela de Classificação Nacional de Atividades Econômicas.
    Cnaes,
    /// Tabela de motivos de situação cadastral.
    Motivos,
    /// Tabela de municípios da Receita Federal / SIAFI.
    Municipios,
    /// Tabela de naturezas jurídicas.
    Naturezas,
    /// Tabela de países segundo a Receita Federal.
    Paises,
    /// Tabela de qualificações de sócios e representantes legais.
    Qualificacoes,
}

impl TableKind {
    /// Converte string para a variante de tabela correspondente.
    ///
    /// ### Parâmetros
    /// - `s`: Identificador textual da tabela (case-insensitive, ex.: `"empresas"`, `"socios"`).
    ///
    /// ### Retorno
    /// Variante `TableKind` mapeada.
    ///
    /// ### Erros
    /// Retorna `ConvertErr::UnsupportedTable` caso o identificador não seja reconhecido.
    pub fn parse(s: &str) -> Result<Self, ConvertErr> {
        match s.trim().to_lowercase().as_str() {
            "empresas" | "empresa" => Ok(Self::Empresas),
            "socios" | "socio" => Ok(Self::Socios),
            "estabelecimentos" | "estabelecimento" => Ok(Self::Estabelecimentos),
            "simples" | "simples_nacional" | "mei" => Ok(Self::Simples),
            "cnaes" | "cnae" => Ok(Self::Cnaes),
            "motivos" | "motivo" => Ok(Self::Motivos),
            "municipios" | "municipio" => Ok(Self::Municipios),
            "naturezas" | "natureza" => Ok(Self::Naturezas),
            "paises" | "pais" => Ok(Self::Paises),
            "qualificacoes" | "qualificacao" => Ok(Self::Qualificacoes),
            _ => Err(ConvertErr::UnsupportedTable(s.to_string())),
        }
    }

    /// Resolve o descritor polimórfico estático correspondente à variante da tabela.
    ///
    /// ### Retorno
    /// Referência imutável a uma implementação estática de `TableDescriptor`.
    pub fn descriptor(&self) -> &'static dyn TableDescriptor {
        match self {
            Self::Empresas => &crate::empresa::EmpresaDescriptor,
            Self::Socios => &crate::socio::SocioDescriptor,
            Self::Estabelecimentos => &crate::estabelecimento::EstabelecimentoDescriptor,
            Self::Simples => &crate::simples::SimplesDescriptor,
            Self::Cnaes => &crate::dominio::CNAES_DESCRIPTOR,
            Self::Motivos => &crate::dominio::MOTIVOS_DESCRIPTOR,
            Self::Municipios => &crate::dominio::MUNICIPIOS_DESCRIPTOR,
            Self::Naturezas => &crate::dominio::NATUREZAS_DESCRIPTOR,
            Self::Paises => &crate::dominio::PAISES_DESCRIPTOR,
            Self::Qualificacoes => &crate::dominio::QUALIFICACOES_DESCRIPTOR,
        }
    }

    /// Retorna o esquema colunar Apache Arrow específico da tabela.
    ///
    /// ### Retorno
    /// Referência compartilhada imutável (`SchemaRef`) para o esquema Arrow configurado.
    pub fn schema(&self) -> SchemaRef {
        self.descriptor().schema()
    }

    /// Constrói um `RecordBatch` para a fatia de bytes de acordo com o tipo da tabela.
    ///
    /// ### Parâmetros
    /// - `slice`: Fatia de bytes mapeada contendo linhas íntegras do CSV.
    ///
    /// ### Retorno
    /// `Ok(Some(RecordBatch))` com as colunas Arrow povoadas ou `Ok(None)` se a fatia estiver vazia.
    ///
    /// ### Erros
    /// Retorna `ConvertErr` caso haja inconsistência de tipos ou falha de construção do batch.
    pub fn build_batch(&self, slice: &[u8]) -> Result<Option<RecordBatch>, ConvertErr> {
        self.descriptor().build_batch(slice)
    }
}

/// Pipeline unificado de conversão de CSV para Parquet em alta vazão.
///
/// ### Parâmetros
/// - `src`: Caminho de origem do arquivo CSV no disco.
/// - `dst`: Caminho de destino do arquivo Parquet colunar gerado.
/// - `kind`: Variante fortemente tipada `TableKind` definindo o parser e esquema.
///
/// ### Retorno
/// Contagem total de linhas válidas convertidas e gravadas com sucesso.
///
/// ### Erros
/// Retorna `ConvertErr` em caso de erro de leitura mapeada, parsing de fatias ou escrita Parquet.
pub fn to_parquet<P1: AsRef<Path>, P2: AsRef<Path>>(
    src: P1,
    dst: P2,
    kind: TableKind,
) -> Result<usize, ConvertErr> {
    let reader: MmapReader = MmapReader::from_path(src)?;
    if reader.is_empty() {
        return Ok(0);
    }

    let num_threads: usize = rayon::current_num_threads().max(1);
    let boundaries: Vec<(usize, usize)> = reader.chunk_boundaries(num_threads);
    let bytes: &[u8] = reader.as_bytes();

    let batches: Vec<RecordBatch> = boundaries
        .par_iter()
        .filter_map(|&(start, end)| {
            let chunk_slice: &[u8] = &bytes[start..end];
            kind.build_batch(chunk_slice).ok().flatten()
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

    let mut writer: ArrowWriter<File> = ArrowWriter::try_new(file, kind.schema(), Some(props))?;
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
    fn test_table_kind_parse() {
        assert_eq!(TableKind::parse("empresas").unwrap(), TableKind::Empresas);
        assert_eq!(TableKind::parse("SOCIOS").unwrap(), TableKind::Socios);
        assert_eq!(
            TableKind::parse("estabelecimento").unwrap(),
            TableKind::Estabelecimentos
        );
        assert_eq!(TableKind::parse("simples").unwrap(), TableKind::Simples);
        assert!(TableKind::parse("invalida").is_err());
    }

    #[test]
    fn test_unified_to_parquet_empresas() {
        let temp_dir: std::path::PathBuf = std::env::temp_dir();
        let csv: std::path::PathBuf = temp_dir.join("test_unified_empresas.csv");
        let parquet: std::path::PathBuf = temp_dir.join("test_unified_empresas.parquet");

        let mut file: File = File::create(&csv).expect("Create temp csv");
        file.write_all(b"\"12ABC345\";\"EMPRESA ALFA\";\"2062\";\"49\";\"15000,00\";\"03\";\"\"\n")
            .expect("Write csv");
        drop(file);

        let rows: usize = to_parquet(&csv, &parquet, TableKind::Empresas).expect("Convert");
        assert_eq!(rows, 1);
        assert!(parquet.exists());

        let _ = std::fs::remove_file(csv);
        let _ = std::fs::remove_file(parquet);
    }

    #[test]
    fn test_unified_to_parquet_socios() {
        let temp_dir: std::path::PathBuf = std::env::temp_dir();
        let csv: std::path::PathBuf = temp_dir.join("test_unified_socios.csv");
        let parquet: std::path::PathBuf = temp_dir.join("test_unified_socios.parquet");

        let mut file: File = File::create(&csv).expect("Create temp csv");
        file.write_all(b"\"12ABC345\";\"2\";\"MARIA SILVA\";\"***123456**\";\"49\";\"20200115\";\"\";\"***000000**\";\"JOSE SILVA\";\"05\";\"5\"\n")
            .expect("Write csv");
        drop(file);

        let rows: usize = to_parquet(&csv, &parquet, TableKind::Socios).expect("Convert");
        assert_eq!(rows, 1);
        assert!(parquet.exists());

        let _ = std::fs::remove_file(csv);
        let _ = std::fs::remove_file(parquet);
    }

    #[test]
    fn test_unified_to_parquet_estabelecimentos() {
        let temp_dir: std::path::PathBuf = std::env::temp_dir();
        let csv: std::path::PathBuf = temp_dir.join("test_unified_estab.csv");
        let parquet: std::path::PathBuf = temp_dir.join("test_unified_estab.parquet");

        let mut file: File = File::create(&csv).expect("Create temp csv");
        file.write_all(b"\"12ABC345\";\"0001\";\"95\";\"1\";\"MATRIZ\";\"02\";\"20210510\";\"00\";\"\";\"\";\"20210510\";\"6201501\";\"\";\"AVENIDA\";\"PAULISTA\";\"1000\";\"SALA 10\";\"BELA VISTA\";\"01310100\";\"SP\";\"7107\";\"11\";\"33334444\";\"\";\"\";\"\";\"\";\"contato@empresa.com\";\"\";\"\"\n")
            .expect("Write csv");
        drop(file);

        let rows: usize = to_parquet(&csv, &parquet, TableKind::Estabelecimentos).expect("Convert");
        assert_eq!(rows, 1);
        assert!(parquet.exists());

        let _ = std::fs::remove_file(csv);
        let _ = std::fs::remove_file(parquet);
    }

    #[test]
    fn test_unified_to_parquet_simples() {
        let temp_dir: std::path::PathBuf = std::env::temp_dir();
        let csv: std::path::PathBuf = temp_dir.join("test_unified_simples.csv");
        let parquet: std::path::PathBuf = temp_dir.join("test_unified_simples.parquet");

        let mut file: File = File::create(&csv).expect("Create temp csv");
        file.write_all(b"\"12ABC345\";\"S\";\"20200101\";\"20211231\";\"N\";\"\";\"\"\n")
            .expect("Write csv");
        drop(file);

        let rows: usize = to_parquet(&csv, &parquet, TableKind::Simples).expect("Convert");
        assert_eq!(rows, 1);
        assert!(parquet.exists());

        let _ = std::fs::remove_file(csv);
        let _ = std::fs::remove_file(parquet);
    }

    #[test]
    fn test_unified_to_parquet_cnaes() {
        let temp_dir: std::path::PathBuf = std::env::temp_dir();
        let csv: std::path::PathBuf = temp_dir.join("test_unified_cnaes.csv");
        let parquet: std::path::PathBuf = temp_dir.join("test_unified_cnaes.parquet");

        let mut file: File = File::create(&csv).expect("Create temp csv");
        file.write_all(b"\"6201501\";\"DESENVOLVIMENTO DE PROGRAMAS\"\n")
            .expect("Write csv");
        drop(file);

        let rows: usize = to_parquet(&csv, &parquet, TableKind::Cnaes).expect("Convert");
        assert_eq!(rows, 1);
        assert!(parquet.exists());

        let _ = std::fs::remove_file(csv);
        let _ = std::fs::remove_file(parquet);
    }

    #[test]
    fn test_unified_to_parquet_municipios() {
        let temp_dir: std::path::PathBuf = std::env::temp_dir();
        let csv: std::path::PathBuf = temp_dir.join("test_unified_municipios.csv");
        let parquet: std::path::PathBuf = temp_dir.join("test_unified_municipios.parquet");

        let mut file: File = File::create(&csv).expect("Create temp csv");
        file.write_all(b"\"7107\";\"SAO PAULO\"\n")
            .expect("Write csv");
        drop(file);

        let rows: usize = to_parquet(&csv, &parquet, TableKind::Municipios).expect("Convert");
        assert_eq!(rows, 1);
        assert!(parquet.exists());

        let _ = std::fs::remove_file(csv);
        let _ = std::fs::remove_file(parquet);
    }
}
