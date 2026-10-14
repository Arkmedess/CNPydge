//! Representação e validação com tipagem forte dos registros da tabela de Empresas do CNPJ.

use crate::converter::{ConvertErr, TableDescriptor};
use crate::util::{ErroCampo, clean_quotes, parse_u8, parse_u16, split_csv_line};
use arrow::array::{ArrayRef, Float64Builder, StringBuilder, UInt8Builder, UInt16Builder};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use std::str;
use std::sync::Arc;

/// Registro validado e fortemente tipado de uma Empresa.
#[derive(Debug, PartialEq)]
pub struct Empresa<'a> {
    /// 8 caracteres alfanuméricos do CNPJ básico.
    pub cnpj: &'a [u8],
    /// Razão social tratada da entidade.
    pub razao: &'a str,
    /// Código numérico da natureza jurídica da entidade.
    pub nat_jur: u16,
    /// Código de qualificação da pessoa física responsável.
    pub qualif: u16,
    /// Capital social em reais (convertido de representação monetária brasileira).
    pub capital: f64,
    /// Código do porte da empresa (01 Não Informado, 03 EPP, 05 Demais).
    pub porte: u8,
    /// Ente federativo responsável no caso de administração pública.
    pub ente_fed: Option<&'a str>,
}

impl<'a> Empresa<'a> {
    /// Faz o parsing e a validação estrita de uma linha CSV da Receita Federal.
    ///
    /// ### Parâmetros
    /// - `linha`: Fatia bruta de bytes referente a uma linha do arquivo CSV de Empresas.
    ///
    /// ### Retorno
    /// Instância de `Empresa<'a>` contendo referências zero-copy aos campos validados.
    ///
    /// ### Erros
    /// - `ErroCampo::CamposInsuficientes`: Se a linha contiver menos de 7 colunas delimitadas por `;`.
    /// - `ErroCampo::CnpjInvalido`: Se o CNPJ básico não possuir 8 posições alfanuméricas.
    /// - `ErroCampo::NumeroInvalido`: Se natureza jurídica, qualificação, capital ou porte forem inválidos.
    /// - `ErroCampo::TextoInvalido`: Se razão social ou ente federativo não forem UTF-8 válidos.
    pub fn parse_line(linha: &'a [u8]) -> Result<Self, ErroCampo> {
        let campos: [&'a [u8]; 7] = split_csv_line::<7>(linha)?;

        let cnpj_raw: &'a [u8] = clean_quotes(campos[0]);
        if cnpj_raw.len() != 8 || !cnpj_raw.iter().all(|b: &u8| b.is_ascii_alphanumeric()) {
            return Err(ErroCampo::CnpjInvalido);
        }

        let razao_bytes: &'a [u8] = clean_quotes(campos[1]);
        let razao: &'a str = str::from_utf8(razao_bytes).map_err(|_| ErroCampo::TextoInvalido)?;

        let nat_jur: u16 = parse_u16(clean_quotes(campos[2]))?;
        let qualif: u16 = parse_u16(clean_quotes(campos[3]))?;
        let capital: f64 = parse_capital(clean_quotes(campos[4]))?;
        let porte: u8 = parse_u8(clean_quotes(campos[5]))?;

        let ente_fed_raw: &'a [u8] = clean_quotes(campos[6]);
        let ente_fed: Option<&'a str> = if ente_fed_raw.is_empty() {
            None
        } else {
            Some(str::from_utf8(ente_fed_raw).map_err(|_| ErroCampo::TextoInvalido)?)
        };

        Ok(Self {
            cnpj: cnpj_raw,
            razao,
            nat_jur,
            qualif,
            capital,
            porte,
            ente_fed,
        })
    }
}

/// Converte valor monetário com vírgula decimal (ex.: "1000,50") em f64.
///
/// ### Parâmetros
/// - `bytes`: Fatia de bytes contendo o valor monetário bruto em formato brasileiro.
///
/// ### Retorno
/// Valor numérico em ponto flutuante de precisão dupla (`f64`).
///
/// ### Erros
/// Retorna `ErroCampo::NumeroInvalido` caso o conteúdo não seja UTF-8 válido ou falhe no parse numérico.
fn parse_capital(bytes: &[u8]) -> Result<f64, ErroCampo> {
    if bytes.is_empty() {
        return Ok(0.0);
    }
    if bytes.len() > 32 {
        return Err(ErroCampo::NumeroInvalido);
    }
    let mut buf = [0u8; 32];
    for (i, &b) in bytes.iter().enumerate() {
        buf[i] = if b == b',' { b'.' } else { b };
    }
    let s = str::from_utf8(&buf[..bytes.len()]).map_err(|_| ErroCampo::NumeroInvalido)?;
    s.parse::<f64>().map_err(|_| ErroCampo::NumeroInvalido)
}

/// Descritor da tabela de Empresas responsável por seu esquema Arrow e montagem de lotes.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmpresaDescriptor;

impl TableDescriptor for EmpresaDescriptor {
    fn name(&self) -> &'static str {
        "empresas"
    }

    fn schema(&self) -> SchemaRef {
        Arc::new(Schema::new(vec![
            Field::new("cnpj_basico", DataType::Utf8, false),
            Field::new("razao_social", DataType::Utf8, false),
            Field::new("natureza_juridica", DataType::UInt16, false),
            Field::new("qualificacao_responsavel", DataType::UInt16, false),
            Field::new("capital_social", DataType::Float64, false),
            Field::new("porte", DataType::UInt8, false),
            Field::new("ente_federativo", DataType::Utf8, true),
        ]))
    }

    fn build_batch(&self, slice: &[u8]) -> Result<Option<RecordBatch>, ConvertErr> {
        if slice.is_empty() {
            return Ok(None);
        }
        let schema: SchemaRef = self.schema();
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
        Ok(Some(RecordBatch::try_new(schema, columns)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_empresa_line() {
        let line: &[u8] =
            b"\"12345678\";\"EMPRESA EXEMPLO LTDA\";\"2062\";\"49\";\"10000,50\";\"03\";\"\"";
        let empresa: Empresa<'_> = Empresa::parse_line(line).expect("Valid line");

        assert_eq!(empresa.cnpj, b"12345678");
        assert_eq!(empresa.razao, "EMPRESA EXEMPLO LTDA");
        assert_eq!(empresa.nat_jur, 2062);
        assert_eq!(empresa.qualif, 49);
        assert_eq!(empresa.capital, 10000.50);
        assert_eq!(empresa.porte, 3);
        assert_eq!(empresa.ente_fed, None);
    }

    #[test]
    fn test_invalid_cnpj_length() {
        let line: &[u8] = b"\"1234\";\"EMPRESA EXEMPLO\";\"2062\";\"49\";\"0,00\";\"01\";\"\"";
        let result: Result<Empresa<'_>, ErroCampo> = Empresa::parse_line(line);
        assert_eq!(result, Err(ErroCampo::CnpjInvalido));
    }

    #[test]
    fn test_missing_fields() {
        let line: &[u8] = b"\"12345678\";\"EMPRESA EXEMPLO\";\"2062\"";
        let result: Result<Empresa<'_>, ErroCampo> = Empresa::parse_line(line);
        assert_eq!(result, Err(ErroCampo::CamposInsuficientes));
    }

    #[test]
    fn test_with_ente_federativo() {
        let line: &[u8] =
            b"\"87654321\";\"ORGAO PUBLICO\";\"1015\";\"05\";\"0,00\";\"05\";\"BRASILIA\"";
        let empresa: Empresa<'_> = Empresa::parse_line(line).expect("Valid line with ente fed");

        assert_eq!(empresa.cnpj, b"87654321");
        assert_eq!(empresa.ente_fed, Some("BRASILIA"));
    }

    #[test]
    fn test_alphanumeric_cnpj() {
        let line: &[u8] =
            b"\"12ABC345\";\"EMPRESA INOVADORA LTDA\";\"2062\";\"49\";\"50000,00\";\"03\";\"\"";
        let empresa: Empresa<'_> = Empresa::parse_line(line).expect("Valid alphanumeric CNPJ");

        assert_eq!(empresa.cnpj, b"12ABC345");
        assert_eq!(empresa.razao, "EMPRESA INOVADORA LTDA");
    }

    #[test]
    fn test_empresa_descriptor_schema_and_batch() {
        let desc = EmpresaDescriptor;
        assert_eq!(desc.name(), "empresas");
        assert_eq!(desc.schema().fields().len(), 7);

        let sample = b"\"12345678\";\"EMPRESA EXEMPLO\";\"2062\";\"49\";\"1000,00\";\"03\";\"\"\n";
        let batch = desc
            .build_batch(sample)
            .expect("build batch")
            .expect("some batch");
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 7);
    }
}
