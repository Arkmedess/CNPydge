//! Representação e validação com tipagem forte dos registros da tabela de Sócios do CNPJ.

use crate::converter::{ConvertErr, TableDescriptor};
use crate::util::{
    ErroCampo, clean_quotes, opt_str, parse_opt_u16, parse_u8, parse_u16, parse_u32, split_csv_line,
};
use arrow::array::{ArrayRef, StringBuilder, UInt8Builder, UInt16Builder, UInt32Builder};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use std::str;
use std::sync::Arc;

/// Registro validado e fortemente tipado de um Sócio da Receita Federal.
#[derive(Debug, PartialEq)]
pub struct Socio<'a> {
    /// 8 caracteres alfanuméricos do CNPJ base.
    pub cnpj: &'a [u8],
    /// Identificador do tipo de sócio (1 PJ, 2 PF, 3 Estrangeiro).
    pub tipo_socio: u8,
    /// Nome do sócio ou razão social da pessoa jurídica sócia.
    pub nome: &'a str,
    /// CPF descaracterizado ou CNPJ do sócio.
    pub doc_socio: &'a str,
    /// Código da qualificação societária (sócio, administrador, etc.).
    pub qualif: u16,
    /// Data de ingresso na sociedade no formato numérico YYYYMMDD.
    pub data_entrada: u32,
    /// Código do país de residência no caso de sócio estrangeiro.
    pub pais: Option<u16>,
    /// CPF descaracterizado do representante legal (se aplicável).
    pub rep_legal: Option<&'a str>,
    /// Nome do representante legal (se aplicável).
    pub nome_rep: Option<&'a str>,
    /// Código de qualificação do representante legal (se aplicável).
    pub qualif_rep: Option<u16>,
    /// Código correspondente à faixa etária do integrante societário (1 a 9).
    pub faixa_etaria: u8,
}

impl<'a> Socio<'a> {
    /// Faz o parsing e a validação estrita de uma linha CSV de Sócios.
    ///
    /// ### Parâmetros
    /// - `line`: Fatia bruta de bytes referente a uma linha do arquivo CSV de Sócios.
    ///
    /// ### Retorno
    /// Instância validada de `Socio<'a>` com referências zero-copy aos campos.
    ///
    /// ### Erros
    /// - `ErroCampo::CamposInsuficientes`: Se a linha contiver menos de 11 colunas delimitadas por `;`.
    /// - `ErroCampo::CnpjInvalido`: Se o CNPJ não possuir 8 posições alfanuméricas.
    /// - `ErroCampo::NumeroInvalido`: Se tipos numéricos ou datas falharem na conversão.
    /// - `ErroCampo::TextoInvalido`: Se strings e documentos não forem UTF-8 válidos.
    pub fn parse_line(line: &'a [u8]) -> Result<Self, ErroCampo> {
        let campos: [&'a [u8]; 11] = split_csv_line::<11>(line)?;

        let cnpj_raw: &'a [u8] = clean_quotes(campos[0]);
        if cnpj_raw.len() != 8 || !cnpj_raw.iter().all(|b: &u8| b.is_ascii_alphanumeric()) {
            return Err(ErroCampo::CnpjInvalido);
        }

        let tipo_socio: u8 = parse_u8(clean_quotes(campos[1]))?;
        let nome_raw: &'a [u8] = clean_quotes(campos[2]);
        let nome: &'a str = str::from_utf8(nome_raw).map_err(|_| ErroCampo::TextoInvalido)?;

        let doc_raw: &'a [u8] = clean_quotes(campos[3]);
        let doc_socio: &'a str = str::from_utf8(doc_raw).map_err(|_| ErroCampo::TextoInvalido)?;

        let qualif: u16 = parse_u16(clean_quotes(campos[4]))?;
        let data_entrada: u32 = parse_u32(clean_quotes(campos[5]))?;
        let pais: Option<u16> = parse_opt_u16(clean_quotes(campos[6]))?;
        let rep_legal: Option<&'a str> = opt_str(clean_quotes(campos[7]))?;
        let nome_rep: Option<&'a str> = opt_str(clean_quotes(campos[8]))?;
        let qualif_rep: Option<u16> = parse_opt_u16(clean_quotes(campos[9]))?;
        let faixa_etaria: u8 = parse_u8(clean_quotes(campos[10]))?;

        Ok(Self {
            cnpj: cnpj_raw,
            tipo_socio,
            nome,
            doc_socio,
            qualif,
            data_entrada,
            pais,
            rep_legal,
            nome_rep,
            qualif_rep,
            faixa_etaria,
        })
    }
}

/// Descritor da tabela de Sócios responsável por seu esquema Arrow e montagem de lotes.
#[derive(Debug, Clone, Copy, Default)]
pub struct SocioDescriptor;

impl TableDescriptor for SocioDescriptor {
    fn name(&self) -> &'static str {
        "socios"
    }

    fn schema(&self) -> SchemaRef {
        Arc::new(Schema::new(vec![
            Field::new("cnpj_basico", DataType::Utf8, false),
            Field::new("identificador_socio", DataType::UInt8, false),
            Field::new("nome_socio", DataType::Utf8, false),
            Field::new("cpf_cnpj_socio", DataType::Utf8, false),
            Field::new("qualificacao_socio", DataType::UInt16, false),
            Field::new("data_entrada_sociedade", DataType::UInt32, false),
            Field::new("pais", DataType::UInt16, true),
            Field::new("representante_legal", DataType::Utf8, true),
            Field::new("nome_representante", DataType::Utf8, true),
            Field::new("qualificacao_representante", DataType::UInt16, true),
            Field::new("faixa_etaria", DataType::UInt8, false),
        ]))
    }

    fn build_batch(&self, slice: &[u8]) -> Result<Option<RecordBatch>, ConvertErr> {
        if slice.is_empty() {
            return Ok(None);
        }
        let schema: SchemaRef = self.schema();
        let mut cnpj_b: StringBuilder = StringBuilder::new();
        let mut tipo_socio_b: UInt8Builder = UInt8Builder::new();
        let mut nome_b: StringBuilder = StringBuilder::new();
        let mut doc_b: StringBuilder = StringBuilder::new();
        let mut qualif_b: UInt16Builder = UInt16Builder::new();
        let mut data_b: UInt32Builder = UInt32Builder::new();
        let mut pais_b: UInt16Builder = UInt16Builder::new();
        let mut rep_legal_b: StringBuilder = StringBuilder::new();
        let mut nome_rep_b: StringBuilder = StringBuilder::new();
        let mut qualif_rep_b: UInt16Builder = UInt16Builder::new();
        let mut faixa_b: UInt8Builder = UInt8Builder::new();
        let mut count: usize = 0;
        let mut start: usize = 0;

        for (i, &byte) in slice.iter().enumerate() {
            if byte == b'\n' {
                let line: &[u8] = &slice[start..i];
                start = i + 1;
                if line.is_empty() || (line.len() == 1 && line[0] == b'\r') {
                    continue;
                }
                let Ok(socio) = Socio::parse_line(line) else {
                    continue;
                };
                let Ok(cnpj_str) = std::str::from_utf8(socio.cnpj) else {
                    continue;
                };

                cnpj_b.append_value(cnpj_str);
                tipo_socio_b.append_value(socio.tipo_socio);
                nome_b.append_value(socio.nome);
                doc_b.append_value(socio.doc_socio);
                qualif_b.append_value(socio.qualif);
                data_b.append_value(socio.data_entrada);
                match socio.pais {
                    Some(p) => pais_b.append_value(p),
                    None => pais_b.append_null(),
                }
                match socio.rep_legal {
                    Some(r) => rep_legal_b.append_value(r),
                    None => rep_legal_b.append_null(),
                }
                match socio.nome_rep {
                    Some(n) => nome_rep_b.append_value(n),
                    None => nome_rep_b.append_null(),
                }
                match socio.qualif_rep {
                    Some(q) => qualif_rep_b.append_value(q),
                    None => qualif_rep_b.append_null(),
                }
                faixa_b.append_value(socio.faixa_etaria);
                count += 1;
            }
        }
        if count == 0 {
            return Ok(None);
        }
        let columns: Vec<ArrayRef> = vec![
            Arc::new(cnpj_b.finish()),
            Arc::new(tipo_socio_b.finish()),
            Arc::new(nome_b.finish()),
            Arc::new(doc_b.finish()),
            Arc::new(qualif_b.finish()),
            Arc::new(data_b.finish()),
            Arc::new(pais_b.finish()),
            Arc::new(rep_legal_b.finish()),
            Arc::new(nome_rep_b.finish()),
            Arc::new(qualif_rep_b.finish()),
            Arc::new(faixa_b.finish()),
        ];
        Ok(Some(RecordBatch::try_new(schema, columns)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_socio_line() {
        let line: &[u8] = b"\"12ABC345\";\"2\";\"MARIA SILVA\";\"***123456**\";\"49\";\"20200115\";\"\";\"***000000**\";\"JOSE SILVA\";\"05\";\"5\"";
        let socio: Socio<'_> = Socio::parse_line(line).expect("Valid socio line");

        assert_eq!(socio.cnpj, b"12ABC345");
        assert_eq!(socio.tipo_socio, 2);
        assert_eq!(socio.nome, "MARIA SILVA");
        assert_eq!(socio.doc_socio, "***123456**");
        assert_eq!(socio.qualif, 49);
        assert_eq!(socio.data_entrada, 20200115);
        assert_eq!(socio.pais, None);
        assert_eq!(socio.rep_legal, Some("***000000**"));
        assert_eq!(socio.nome_rep, Some("JOSE SILVA"));
        assert_eq!(socio.qualif_rep, Some(5));
        assert_eq!(socio.faixa_etaria, 5);
    }

    #[test]
    fn test_socio_missing_fields() {
        let line: &[u8] = b"\"12ABC345\";\"2\";\"MARIA SILVA\"";
        let result: Result<Socio<'_>, ErroCampo> = Socio::parse_line(line);
        assert_eq!(result, Err(ErroCampo::CamposInsuficientes));
    }

    #[test]
    fn test_socio_invalid_cnpj() {
        let line: &[u8] = b"\"123\";\"2\";\"MARIA SILVA\";\"***123456**\";\"49\";\"20200115\";\"\";\"\";\"\";\"\";\"5\"";
        let result: Result<Socio<'_>, ErroCampo> = Socio::parse_line(line);
        assert_eq!(result, Err(ErroCampo::CnpjInvalido));
    }

    #[test]
    fn test_socio_descriptor_schema_and_batch() {
        let desc = SocioDescriptor;
        assert_eq!(desc.name(), "socios");
        assert_eq!(desc.schema().fields().len(), 11);

        let sample = b"\"12ABC345\";\"2\";\"MARIA SILVA\";\"***123456**\";\"49\";\"20200115\";\"\";\"***000000**\";\"JOSE SILVA\";\"05\";\"5\"\n";
        let batch = desc
            .build_batch(sample)
            .expect("build batch")
            .expect("some batch");
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 11);
    }
}
