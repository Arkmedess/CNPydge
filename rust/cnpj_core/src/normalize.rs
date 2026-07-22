//! Record normalization.
//!
//! Converts raw parsed fields into a `NormalizedRecord` with fixed-size
//! CNPJ and trimmed/validated fields. Zero-copy where possible: fields
//! are moved from the parsed Vec into the record, not cloned.
//!
//! The RFB Empresas file has this layout (pipe-delimited, 0-indexed):
///
/// ```text
///  0: CNPJ raiz (8 bytes)
///  1: CNPJ ordem (4 bytes)
///  2: CNPJ DV (2 bytes)
///  3: Identificador Matriz/Filial (1 byte)
///  4: Razao Social (150 bytes max)
///  5: CNAE Fiscal Principal (7 bytes)
///  6: Descricao Natureza Juridica (varies)
///  7: Qualificacao do Representante Legal (2 bytes)
///  8: Nome do Representante Legal (varies)
///  9: Codigo do Pais do Representante Legal (varies)
/// 10: Opcao pelo Simples Nacional (1 byte)
/// 11: Data de Opcao pelo Simples (8 bytes)
/// 12: Data de Exclusao do Simples (8 bytes)
/// 13: Opcao pelo MEI (1 byte)
/// 14: Situacao Cadastral (2 bytes)
/// 15: Data da Situacao Cadastral (8 bytes)
/// 16: Codigo do Motivo da Situacao Cadastral (varies)
/// 17: Nome Exterior do Pais (varies)
/// 18: Codigo do Pais (varies)
/// 19: Natureza Juridica (4 bytes)
/// 20: CNAE Fiscal Secundaria (varies)
/// 21: Codigo do Municipio (7 bytes)
/// 22: Codigo do DDD 1 (2 bytes)
/// 23: Telefone 1 (varies)
/// 24: Codigo do DDD 2 (2 bytes)
/// 25: Telefone 2 (varies)
/// 26: Codigo do DDD do Fax (2 bytes)
/// 27: Numero do Fax (varies)
/// 28: Correspondencia (1 byte)
/// 29: CEP (8 bytes)
/// 30: Bairro (varies)
/// 31: Cidade (varies)
/// 32: Sigla da Unidade Federativa (2 bytes)
/// 33: Codigo do Municipio (7 bytes) [duplicate?]
/// 34: Situacao Especial (2 bytes)
/// 35: Data da Situacao Especial (8 bytes)
/// 36: Capital Social (varies, decimal)
/// ```

use crate::channels::NormalizedRecord;

/// Number of fields expected in the Empresas CSV.
/// We accept fewer (some fields may be missing at end of line).
pub const EXPECTED_FIELDS_EMPRESAS: usize = 37;

/// Normalize a parsed record (vector of byte-vecs) into a `NormalizedRecord`.
///
/// # Errors
/// Returns `None` if the record has fewer than the minimum required fields
/// or if the CNPJ fields are invalid.
pub fn normalize_empresas(fields: &[Vec<u8>]) -> Option<NormalizedRecord> {
    // Minimum fields needed: CNPJ raiz(0) + ordem(1) + dv(2) + a few more
    if fields.len() < 16 {
        return None;
    }

    // Build CNPJ: raiz(8) + ordem(4) + dv(2) = 14 bytes
    let raiz = &fields[0];
    let ordem = &fields[1];
    let dv = &fields[2];

    if raiz.len() != 8 || ordem.len() != 4 || dv.len() != 2 {
        return None;
    }

    let mut cnpj = [0u8; 14];
    cnpj[0..8].copy_from_slice(raiz);
    cnpj[8..12].copy_from_slice(ordem);
    cnpj[12..14].copy_from_slice(dv);

    Some(NormalizedRecord {
        cnpj,
        razao_social: fields[4].clone(),
        cnae_fiscal: fields[5].clone(),
        situacao_cadastral: fields[14].clone(),
        data_situacao_cadastral: fields[15].clone(),
        uf: get_field(fields, 32),
        codigo_municipio: get_field(fields, 21),
        cep: get_field(fields, 29),
        ddd_telefone_1: get_field(fields, 22),
        ddd_telefone_2: get_field(fields, 24),
        ddd_fax: get_field(fields, 26),
        data_abertura: get_field(fields, 3),
        natureza_juridica: get_field(fields, 19),
        qualificacao_representante_legal: get_field(fields, 7),
        porte_empresa: Vec::new(), // Not in Empresas, may be in Estabelecimentos
        opcao_simples_nacional: get_field(fields, 10),
        data_opcao_simples_nacional: get_field(fields, 11),
        data_exclusao_simples_nacional: get_field(fields, 12),
        opcao_mei: get_field(fields, 13),
        situacao_especial: get_field(fields, 34),
        data_situacao_especial: get_field(fields, 35),
        capital_social: get_field(fields, 36),
    })
}

/// Safely get a field by index, returning an empty Vec if out of bounds.
#[inline]
fn get_field(fields: &[Vec<u8>], index: usize) -> Vec<u8> {
    if index < fields.len() {
        fields[index].clone()
    } else {
        Vec::new()
    }
}

/// Trim whitespace from a byte slice.
#[inline]
pub fn trim_bytes(data: &[u8]) -> &[u8] {
    if data.is_empty() {
        return data;
    }
    let start = data.iter().position(|&b| b != b' ' && b != b'\t').unwrap_or(data.len());
    if start == data.len() {
        return &[];
    }
    let end = data.iter().rposition(|&b| b != b' ' && b != b'\t').map_or(0, |i| i + 1);
    &data[start..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_empresas_basic() {
        let fields: Vec<Vec<u8>> = vec![
            b"12345678".to_vec(),        // 0: raiz
            b"0001".to_vec(),             // 1: ordem
            b"95".to_vec(),               // 2: dv
            b"20200101".to_vec(),         // 3: data abertura
            b"Empresa Teste".to_vec(),    // 4: razao social
            b"6201501".to_vec(),          // 5: cnae fiscal
            b"".to_vec(),                 // 6: descricao natureza juridica
            b"49".to_vec(),               // 7: qualificacao representante
            b"".to_vec(),                 // 8: nome representante
            b"".to_vec(),                 // 9: codigo pais representante
            b"N".to_vec(),                // 10: simples nacional
            b"".to_vec(),                 // 11: data opcao simples
            b"".to_vec(),                 // 12: data exclusao simples
            b"N".to_vec(),                // 13: opcao mei
            b"02".to_vec(),               // 14: situacao cadastral
            b"20200101".to_vec(),         // 15: data situacao cadastral
        ];

        let record = normalize_empresas(&fields).unwrap();
        assert_eq!(record.cnpj, *b"12345678000195");
        assert_eq!(record.razao_social, b"Empresa Teste");
        assert_eq!(record.cnae_fiscal, b"6201501");
        assert_eq!(record.situacao_cadastral, b"02");
    }

    #[test]
    fn test_normalize_too_few_fields() {
        let fields: Vec<Vec<u8>> = vec![b"12345678".to_vec(), b"0001".to_vec()];
        assert!(normalize_empresas(&fields).is_none());
    }

    #[test]
    fn test_trim_bytes() {
        assert_eq!(trim_bytes(b"  hello  "), b"hello");
        assert_eq!(trim_bytes(b"no_trim"), b"no_trim");
        assert_eq!(trim_bytes(b"   "), b"");
    }
}
