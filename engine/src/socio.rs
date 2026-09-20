//! Representação e validação com tipagem forte dos registros da tabela de Sócios do CNPJ.

use crate::util::{
    ErroCampo, clean_quotes, opt_str, parse_opt_u16, parse_u8, parse_u16, parse_u32,
};
use std::str;

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
        let mut campos: [&'a [u8]; 11] = [&[]; 11];
        let mut indice_campo: usize = 0;
        let mut inicio: usize = 0;
        let mut em_aspas: bool = false;

        for (i, &byte) in line.iter().enumerate() {
            if byte == b'"' {
                em_aspas = !em_aspas;
            } else if byte == b';' && !em_aspas {
                if indice_campo < 11 {
                    campos[indice_campo] = &line[inicio..i];
                    indice_campo += 1;
                }
                inicio = i + 1;
            }
        }

        if indice_campo < 11 && inicio <= line.len() {
            let mut fim: usize = line.len();
            while fim > inicio && (line[fim - 1] == b'\n' || line[fim - 1] == b'\r') {
                fim -= 1;
            }
            campos[indice_campo] = &line[inicio..fim];
            indice_campo += 1;
        }

        if indice_campo != 11 {
            return Err(ErroCampo::CamposInsuficientes);
        }

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
}
