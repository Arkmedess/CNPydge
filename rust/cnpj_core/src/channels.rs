//! Crossbeam channel topology for the pipeline.
//!
//! Each stage communicates via bounded channels for natural backpressure.
//! The topology is:
//!
//! ```text
//! [Reader] -> bounded -> [Parser(s)] -> bounded -> [Batcher] -> bounded -> [Writer(s)]
//! ```

use crossbeam_channel::{bounded, Receiver, Sender};

/// Capacity of each bounded channel (number of items in flight).
/// Keeps memory constant: a few batches in flight, not thousands.
pub const CHANNEL_CAPACITY: usize = 8;

/// A raw CSV line (bytes from mmap, not yet parsed).
pub type RawLine = Vec<u8>;

/// A parsed but not-yet-normalized record (fields as byte slices).
/// `'static` because the data is copied out of the mmap into owned Vecs
/// for thread safety across the channel boundary.
pub type ParsedRecord = Vec<Vec<u8>>;

/// A batch of normalized records ready for writing.
pub type Batch = Vec<NormalizedRecord>;

/// A single normalized record with fixed-size fields.
/// All fields are stored as bytes to avoid String allocation in the hot path.
#[derive(Debug, Clone)]
pub struct NormalizedRecord {
    /// CNPJ completo (14 chars: raiz[8] + ordem[4] + dv[2])
    pub cnpj: [u8; 14],
    /// Razao social (trim, max 150 bytes)
    pub razao_social: Vec<u8>,
    /// CNAE fiscal principal (7 chars)
    pub cnae_fiscal: Vec<u8>,
    /// Situacao cadastral (2 chars)
    pub situacao_cadastral: Vec<u8>,
    /// Data da situacao cadastral (8 chars: YYYYMMDD)
    pub data_situacao_cadastral: Vec<u8>,
    /// UF (2 chars)
    pub uf: Vec<u8>,
    /// Codigo do municipio (7 chars)
    pub codigo_municipio: Vec<u8>,
    /// CEP (8 chars)
    pub cep: Vec<u8>,
    /// DDD telefone 1 (2 chars)
    pub ddd_telefone_1: Vec<u8>,
    /// DDD telefone 2 (2 chars)
    pub ddd_telefone_2: Vec<u8>,
    /// DDD fax (2 chars)
    pub ddd_fax: Vec<u8>,
    /// Data de abertura (8 chars: YYYYMMDD)
    pub data_abertura: Vec<u8>,
    /// Natureza juridica (4 chars)
    pub natureza_juridica: Vec<u8>,
    /// Qualificacao do representante legal (2 chars)
    pub qualificacao_representante_legal: Vec<u8>,
    /// Porte da empresa (2 chars)
    pub porte_empresa: Vec<u8>,
    /// Opcao pelo Simples Nacional (1 char: S/N)
    pub opcao_simples_nacional: Vec<u8>,
    /// Data de opcao pelo Simples (8 chars)
    pub data_opcao_simples_nacional: Vec<u8>,
    /// Exclusao do Simples (8 chars)
    pub data_exclusao_simples_nacional: Vec<u8>,
    /// Opcao pelo MEI (1 char: S/N)
    pub opcao_mei: Vec<u8>,
    /// Situacao especial (2 chars)
    pub situacao_especial: Vec<u8>,
    /// Data da situacao especial (8 chars)
    pub data_situacao_especial: Vec<u8>,
    /// Capital social (15 chars, decimal)
    pub capital_social: Vec<u8>,
}

/// Create the reader -> parser channel pair.
pub fn raw_line_channel() -> (Sender<RawLine>, Receiver<RawLine>) {
    bounded(CHANNEL_CAPACITY)
}

/// Create the parser -> batcher channel pair (sends NormalizedRecord).
pub fn normalized_record_channel() -> (Sender<NormalizedRecord>, Receiver<NormalizedRecord>) {
    bounded(CHANNEL_CAPACITY)
}

/// Create the batcher -> writer channel pair.
pub fn batch_channel() -> (Sender<Batch>, Receiver<Batch>) {
    bounded(CHANNEL_CAPACITY)
}
