//! Representação e validação com tipagem forte dos registros da tabela de Estabelecimentos do CNPJ.

use crate::converter::{ConvertErr, TableDescriptor};
use crate::util::{
    ErroCampo, clean_quotes, opt_str, parse_opt_u16, parse_opt_u32, parse_u8, parse_u16, parse_u32,
    split_csv_line,
};
use arrow::array::{ArrayRef, StringBuilder, UInt8Builder, UInt16Builder, UInt32Builder};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use std::str;
use std::sync::Arc;

/// Registro validado e fortemente tipado de um Estabelecimento da Receita Federal.
#[derive(Debug, PartialEq)]
pub struct Estabelecimento<'a> {
    /// 8 caracteres alfanuméricos do CNPJ básico.
    pub cnpj_basico: &'a [u8],
    /// 4 caracteres da ordem do estabelecimento (ex.: "0001").
    pub cnpj_ordem: &'a [u8],
    /// 2 caracteres do dígito verificador do CNPJ.
    pub cnpj_dv: &'a [u8],
    /// Identificador de tipo (1 Matriz, 2 Filial).
    pub matriz_filial: u8,
    /// Nome fantasia do estabelecimento (se houver).
    pub fantasia: Option<&'a str>,
    /// Código da situação cadastral (01 Nula, 02 Ativa, 03 Suspensa, 04 Inapta, 08 Baixada).
    pub situacao: u8,
    /// Data do evento da situação cadastral no formato YYYYMMDD.
    pub data_situacao: Option<u32>,
    /// Código do motivo da situação cadastral.
    pub motivo_situacao: u16,
    /// Nome da cidade no exterior (para unidades sediadas fora do país).
    pub cidade_exterior: Option<&'a str>,
    /// Código do país no exterior.
    pub pais: Option<u16>,
    /// Data de início da atividade econômica no formato YYYYMMDD.
    pub data_inicio: Option<u32>,
    /// Código numérico da atividade econômica fiscal principal (CNAE de 7 dígitos).
    pub cnae_principal: u32,
    /// Lista de códigos CNAEs secundários separados por vírgula.
    pub cnae_secundario: Option<&'a str>,
    /// Tipo de logradouro (Rua, Avenida, etc.).
    pub tipo_logradouro: Option<&'a str>,
    /// Nome do logradouro.
    pub logradouro: Option<&'a str>,
    /// Número do imóvel ou identificação predial.
    pub numero: Option<&'a str>,
    /// Complemento do endereço (sala, bloco, etc.).
    pub complemento: Option<&'a str>,
    /// Bairro ou distrito do endereço.
    pub bairro: Option<&'a str>,
    /// Código de Endereçamento Postal (8 caracteres).
    pub cep: Option<&'a str>,
    /// Sigla da Unidade Federativa estadual (ex.: "SP").
    pub uf: Option<&'a str>,
    /// Código numérico de identificação municipal (SIAFI/RFB).
    pub municipio: Option<u16>,
    /// Código DDD do telefone primário.
    pub ddd_1: Option<&'a str>,
    /// Número do telefone primário.
    pub telefone_1: Option<&'a str>,
    /// Código DDD do telefone secundário.
    pub ddd_2: Option<&'a str>,
    /// Número do telefone secundário.
    pub telefone_2: Option<&'a str>,
    /// Código DDD do fax.
    pub ddd_fax: Option<&'a str>,
    /// Número do fax.
    pub fax: Option<&'a str>,
    /// Endereço de correio eletrônico cadastrado.
    pub email: Option<&'a str>,
    /// Situação especial (recuperação judicial, falência, etc.).
    pub situacao_especial: Option<&'a str>,
    /// Data da ocorrência da situação especial (YYYYMMDD).
    pub data_situacao_especial: Option<u32>,
}

impl<'a> Estabelecimento<'a> {
    /// Faz o parsing e a validação estrita de uma linha CSV de Estabelecimento.
    ///
    /// ### Parâmetros
    /// - `line`: Fatia bruta de bytes referente a uma linha do arquivo CSV de Estabelecimentos.
    ///
    /// ### Retorno
    /// Instância validada de `Estabelecimento<'a>` contendo referências zero-copy.
    ///
    /// ### Erros
    /// - `ErroCampo::CamposInsuficientes`: Se a linha contiver menos de 30 colunas delimitadas por `;`.
    /// - `ErroCampo::CnpjInvalido`: Se as fatias do CNPJ violarem os tamanhos ou padrão alfanumérico.
    /// - `ErroCampo::NumeroInvalido`: Se campos inteiros ou datas falharem na conversão numérica.
    /// - `ErroCampo::TextoInvalido`: Se os campos textuais não forem UTF-8 válidos.
    pub fn parse_line(line: &'a [u8]) -> Result<Self, ErroCampo> {
        let campos: [&'a [u8]; 30] = split_csv_line::<30>(line)?;

        let cnpj_basico: &'a [u8] = clean_quotes(campos[0]);
        if cnpj_basico.len() != 8 || !cnpj_basico.iter().all(|b: &u8| b.is_ascii_alphanumeric()) {
            return Err(ErroCampo::CnpjInvalido);
        }

        let cnpj_ordem: &'a [u8] = clean_quotes(campos[1]);
        let cnpj_dv: &'a [u8] = clean_quotes(campos[2]);
        let matriz_filial: u8 = parse_u8(clean_quotes(campos[3]))?;
        let fantasia: Option<&'a str> = opt_str(clean_quotes(campos[4]))?;
        let situacao: u8 = parse_u8(clean_quotes(campos[5]))?;
        let data_situacao: Option<u32> = parse_opt_u32(clean_quotes(campos[6]))?;
        let motivo_situacao: u16 = parse_u16(clean_quotes(campos[7]))?;
        let cidade_exterior: Option<&'a str> = opt_str(clean_quotes(campos[8]))?;
        let pais: Option<u16> = parse_opt_u16(clean_quotes(campos[9]))?;
        let data_inicio: Option<u32> = parse_opt_u32(clean_quotes(campos[10]))?;
        let cnae_principal: u32 = parse_u32(clean_quotes(campos[11]))?;
        let cnae_secundario: Option<&'a str> = opt_str(clean_quotes(campos[12]))?;
        let tipo_logradouro: Option<&'a str> = opt_str(clean_quotes(campos[13]))?;
        let logradouro: Option<&'a str> = opt_str(clean_quotes(campos[14]))?;
        let numero: Option<&'a str> = opt_str(clean_quotes(campos[15]))?;
        let complemento: Option<&'a str> = opt_str(clean_quotes(campos[16]))?;
        let bairro: Option<&'a str> = opt_str(clean_quotes(campos[17]))?;
        let cep: Option<&'a str> = opt_str(clean_quotes(campos[18]))?;
        let uf: Option<&'a str> = opt_str(clean_quotes(campos[19]))?;
        let municipio: Option<u16> = parse_opt_u16(clean_quotes(campos[20]))?;
        let ddd_1: Option<&'a str> = opt_str(clean_quotes(campos[21]))?;
        let telefone_1: Option<&'a str> = opt_str(clean_quotes(campos[22]))?;
        let ddd_2: Option<&'a str> = opt_str(clean_quotes(campos[23]))?;
        let telefone_2: Option<&'a str> = opt_str(clean_quotes(campos[24]))?;
        let ddd_fax: Option<&'a str> = opt_str(clean_quotes(campos[25]))?;
        let fax: Option<&'a str> = opt_str(clean_quotes(campos[26]))?;
        let email: Option<&'a str> = opt_str(clean_quotes(campos[27]))?;
        let situacao_especial: Option<&'a str> = opt_str(clean_quotes(campos[28]))?;
        let data_situacao_especial: Option<u32> = parse_opt_u32(clean_quotes(campos[29]))?;

        Ok(Self {
            cnpj_basico,
            cnpj_ordem,
            cnpj_dv,
            matriz_filial,
            fantasia,
            situacao,
            data_situacao,
            motivo_situacao,
            cidade_exterior,
            pais,
            data_inicio,
            cnae_principal,
            cnae_secundario,
            tipo_logradouro,
            logradouro,
            numero,
            complemento,
            bairro,
            cep,
            uf,
            municipio,
            ddd_1,
            telefone_1,
            ddd_2,
            telefone_2,
            ddd_fax,
            fax,
            email,
            situacao_especial,
            data_situacao_especial,
        })
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
fn append_opt_u16(b: &mut UInt16Builder, val: Option<u16>) {
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

/// Descritor da tabela de Estabelecimentos responsável pelo esquema Arrow de 30 colunas e montagem de lotes.
#[derive(Debug, Clone, Copy, Default)]
pub struct EstabelecimentoDescriptor;

impl TableDescriptor for EstabelecimentoDescriptor {
    fn name(&self) -> &'static str {
        "estabelecimentos"
    }

    fn schema(&self) -> SchemaRef {
        Arc::new(Schema::new(vec![
            Field::new("cnpj_basico", DataType::Utf8, false),
            Field::new("cnpj_ordem", DataType::Utf8, false),
            Field::new("cnpj_dv", DataType::Utf8, false),
            Field::new("matriz_filial", DataType::UInt8, false),
            Field::new("nome_fantasia", DataType::Utf8, true),
            Field::new("situacao_cadastral", DataType::UInt8, false),
            Field::new("data_situacao_cadastral", DataType::UInt32, true),
            Field::new("motivo_situacao_cadastral", DataType::UInt16, false),
            Field::new("nome_cidade_exterior", DataType::Utf8, true),
            Field::new("pais", DataType::UInt16, true),
            Field::new("data_inicio_atividade", DataType::UInt32, true),
            Field::new("cnae_fiscal_principal", DataType::UInt32, false),
            Field::new("cnae_fiscal_secundaria", DataType::Utf8, true),
            Field::new("tipo_logradouro", DataType::Utf8, true),
            Field::new("logradouro", DataType::Utf8, true),
            Field::new("numero", DataType::Utf8, true),
            Field::new("complemento", DataType::Utf8, true),
            Field::new("bairro", DataType::Utf8, true),
            Field::new("cep", DataType::Utf8, true),
            Field::new("uf", DataType::Utf8, true),
            Field::new("municipio", DataType::UInt16, true),
            Field::new("ddd_1", DataType::Utf8, true),
            Field::new("telefone_1", DataType::Utf8, true),
            Field::new("ddd_2", DataType::Utf8, true),
            Field::new("telefone_2", DataType::Utf8, true),
            Field::new("ddd_fax", DataType::Utf8, true),
            Field::new("fax", DataType::Utf8, true),
            Field::new("correio_eletronico", DataType::Utf8, true),
            Field::new("situacao_especial", DataType::Utf8, true),
            Field::new("data_situacao_especial", DataType::UInt32, true),
        ]))
    }

    fn build_batch(&self, slice: &[u8]) -> Result<Option<RecordBatch>, ConvertErr> {
        if slice.is_empty() {
            return Ok(None);
        }
        let schema: SchemaRef = self.schema();
        let mut cnpj_basico_b: StringBuilder = StringBuilder::new();
        let mut cnpj_ordem_b: StringBuilder = StringBuilder::new();
        let mut cnpj_dv_b: StringBuilder = StringBuilder::new();
        let mut matriz_filial_b: UInt8Builder = UInt8Builder::new();
        let mut fantasia_b: StringBuilder = StringBuilder::new();
        let mut situacao_b: UInt8Builder = UInt8Builder::new();
        let mut data_situacao_b: UInt32Builder = UInt32Builder::new();
        let mut motivo_b: UInt16Builder = UInt16Builder::new();
        let mut cidade_ext_b: StringBuilder = StringBuilder::new();
        let mut pais_b: UInt16Builder = UInt16Builder::new();
        let mut data_inicio_b: UInt32Builder = UInt32Builder::new();
        let mut cnae_princ_b: UInt32Builder = UInt32Builder::new();
        let mut cnae_sec_b: StringBuilder = StringBuilder::new();
        let mut tipo_logr_b: StringBuilder = StringBuilder::new();
        let mut logradouro_b: StringBuilder = StringBuilder::new();
        let mut numero_b: StringBuilder = StringBuilder::new();
        let mut complemento_b: StringBuilder = StringBuilder::new();
        let mut bairro_b: StringBuilder = StringBuilder::new();
        let mut cep_b: StringBuilder = StringBuilder::new();
        let mut uf_b: StringBuilder = StringBuilder::new();
        let mut municipio_b: UInt16Builder = UInt16Builder::new();
        let mut ddd_1_b: StringBuilder = StringBuilder::new();
        let mut tel_1_b: StringBuilder = StringBuilder::new();
        let mut ddd_2_b: StringBuilder = StringBuilder::new();
        let mut tel_2_b: StringBuilder = StringBuilder::new();
        let mut ddd_fax_b: StringBuilder = StringBuilder::new();
        let mut fax_b: StringBuilder = StringBuilder::new();
        let mut email_b: StringBuilder = StringBuilder::new();
        let mut sit_esp_b: StringBuilder = StringBuilder::new();
        let mut data_sit_esp_b: UInt32Builder = UInt32Builder::new();

        let mut count: usize = 0;
        let mut start: usize = 0;

        for (i, &byte) in slice.iter().enumerate() {
            if byte == b'\n' {
                let line: &[u8] = &slice[start..i];
                start = i + 1;
                if line.is_empty() || (line.len() == 1 && line[0] == b'\r') {
                    continue;
                }
                let Ok(est) = Estabelecimento::parse_line(line) else {
                    continue;
                };
                let Ok(cnpj_str) = std::str::from_utf8(est.cnpj_basico) else {
                    continue;
                };
                let Ok(ordem_str) = std::str::from_utf8(est.cnpj_ordem) else {
                    continue;
                };
                let Ok(dv_str) = std::str::from_utf8(est.cnpj_dv) else {
                    continue;
                };

                cnpj_basico_b.append_value(cnpj_str);
                cnpj_ordem_b.append_value(ordem_str);
                cnpj_dv_b.append_value(dv_str);
                matriz_filial_b.append_value(est.matriz_filial);

                append_opt_str(&mut fantasia_b, est.fantasia);
                situacao_b.append_value(est.situacao);
                append_opt_u32(&mut data_situacao_b, est.data_situacao);
                motivo_b.append_value(est.motivo_situacao);
                append_opt_str(&mut cidade_ext_b, est.cidade_exterior);
                append_opt_u16(&mut pais_b, est.pais);
                append_opt_u32(&mut data_inicio_b, est.data_inicio);
                cnae_princ_b.append_value(est.cnae_principal);
                append_opt_str(&mut cnae_sec_b, est.cnae_secundario);
                append_opt_str(&mut tipo_logr_b, est.tipo_logradouro);
                append_opt_str(&mut logradouro_b, est.logradouro);
                append_opt_str(&mut numero_b, est.numero);
                append_opt_str(&mut complemento_b, est.complemento);
                append_opt_str(&mut bairro_b, est.bairro);
                append_opt_str(&mut cep_b, est.cep);
                append_opt_str(&mut uf_b, est.uf);
                append_opt_u16(&mut municipio_b, est.municipio);
                append_opt_str(&mut ddd_1_b, est.ddd_1);
                append_opt_str(&mut tel_1_b, est.telefone_1);
                append_opt_str(&mut ddd_2_b, est.ddd_2);
                append_opt_str(&mut tel_2_b, est.telefone_2);
                append_opt_str(&mut ddd_fax_b, est.ddd_fax);
                append_opt_str(&mut fax_b, est.fax);
                append_opt_str(&mut email_b, est.email);
                append_opt_str(&mut sit_esp_b, est.situacao_especial);
                append_opt_u32(&mut data_sit_esp_b, est.data_situacao_especial);

                count += 1;
            }
        }
        if count == 0 {
            return Ok(None);
        }
        let columns: Vec<ArrayRef> = vec![
            Arc::new(cnpj_basico_b.finish()),
            Arc::new(cnpj_ordem_b.finish()),
            Arc::new(cnpj_dv_b.finish()),
            Arc::new(matriz_filial_b.finish()),
            Arc::new(fantasia_b.finish()),
            Arc::new(situacao_b.finish()),
            Arc::new(data_situacao_b.finish()),
            Arc::new(motivo_b.finish()),
            Arc::new(cidade_ext_b.finish()),
            Arc::new(pais_b.finish()),
            Arc::new(data_inicio_b.finish()),
            Arc::new(cnae_princ_b.finish()),
            Arc::new(cnae_sec_b.finish()),
            Arc::new(tipo_logr_b.finish()),
            Arc::new(logradouro_b.finish()),
            Arc::new(numero_b.finish()),
            Arc::new(complemento_b.finish()),
            Arc::new(bairro_b.finish()),
            Arc::new(cep_b.finish()),
            Arc::new(uf_b.finish()),
            Arc::new(municipio_b.finish()),
            Arc::new(ddd_1_b.finish()),
            Arc::new(tel_1_b.finish()),
            Arc::new(ddd_2_b.finish()),
            Arc::new(tel_2_b.finish()),
            Arc::new(ddd_fax_b.finish()),
            Arc::new(fax_b.finish()),
            Arc::new(email_b.finish()),
            Arc::new(sit_esp_b.finish()),
            Arc::new(data_sit_esp_b.finish()),
        ];
        Ok(Some(RecordBatch::try_new(schema, columns)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_estabelecimento_line() {
        let line: &[u8] = b"\"12ABC345\";\"0001\";\"95\";\"1\";\"FANTASIA MATRIZ\";\"02\";\"20210510\";\"00\";\"\";\"\";\"20210510\";\"6201501\";\"\";\"AVENIDA\";\"PAULISTA\";\"1000\";\"SALA 10\";\"BELA VISTA\";\"01310100\";\"SP\";\"7107\";\"11\";\"33334444\";\"\";\"\";\"\";\"\";\"contato@empresa.com\";\"\";\"\"";
        let est: Estabelecimento<'_> =
            Estabelecimento::parse_line(line).expect("Valid estabelecimento line");

        assert_eq!(est.cnpj_basico, b"12ABC345");
        assert_eq!(est.cnpj_ordem, b"0001");
        assert_eq!(est.cnpj_dv, b"95");
        assert_eq!(est.matriz_filial, 1);
        assert_eq!(est.fantasia, Some("FANTASIA MATRIZ"));
        assert_eq!(est.situacao, 2);
        assert_eq!(est.cnae_principal, 6201501);
        assert_eq!(est.uf, Some("SP"));
        assert_eq!(est.municipio, Some(7107));
    }

    #[test]
    fn test_estabelecimento_insufficient_fields() {
        let line: &[u8] = b"\"12ABC345\";\"0001\";\"95\";\"1\"";
        let result: Result<Estabelecimento<'_>, ErroCampo> = Estabelecimento::parse_line(line);
        assert_eq!(result, Err(ErroCampo::CamposInsuficientes));
    }

    #[test]
    fn test_estabelecimento_descriptor_schema_and_batch() {
        let desc = EstabelecimentoDescriptor;
        assert_eq!(desc.name(), "estabelecimentos");
        assert_eq!(desc.schema().fields().len(), 30);

        let sample = b"\"12ABC345\";\"0001\";\"95\";\"1\";\"MATRIZ\";\"02\";\"20210510\";\"00\";\"\";\"\";\"20210510\";\"6201501\";\"\";\"AVENIDA\";\"PAULISTA\";\"1000\";\"SALA 10\";\"BELA VISTA\";\"01310100\";\"SP\";\"7107\";\"11\";\"33334444\";\"\";\"\";\"\";\"\";\"contato@empresa.com\";\"\";\"\"\n";
        let batch = desc
            .build_batch(sample)
            .expect("build batch")
            .expect("some batch");
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 30);
    }
}
