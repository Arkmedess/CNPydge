//! Representação e validação com tipagem forte dos registros da tabela de Empresas do CNPJ.

use crate::util::{ErroCampo, clean_quotes, parse_u8, parse_u16};
use std::str;

/// Registro validado e fortemente tipado de uma Empresa.
#[derive(Debug, PartialEq)]
pub struct Empresa<'a> {
    pub cnpj: &'a [u8],            // 8 dígitos alfanuméricos do CNPJ básico
    pub razao: &'a str,            // Razão social tratada
    pub nat_jur: u16,              // Código da natureza jurídica
    pub qualif: u16,               // Código de qualificação do responsável
    pub capital: f64,              // Capital social em reais
    pub porte: u8,                 // Código do porte da empresa
    pub ente_fed: Option<&'a str>, // Ente federativo responsável
}

impl<'a> Empresa<'a> {
    /// Faz o parsing e a validação estrita de uma linha CSV da Receita Federal.
    pub fn parse_line(linha: &'a [u8]) -> Result<Self, ErroCampo> {
        let mut campos: [&'a [u8]; 7] = [&[]; 7];
        let mut indice_campo: usize = 0;
        let mut inicio: usize = 0;
        let mut em_aspas: bool = false;

        for (i, &byte) in linha.iter().enumerate() {
            if byte == b'"' {
                em_aspas = !em_aspas;
            } else if byte == b';' && !em_aspas {
                if indice_campo < 7 {
                    campos[indice_campo] = &linha[inicio..i];
                    indice_campo += 1;
                }
                inicio = i + 1;
            }
        }

        if indice_campo < 7 && inicio <= linha.len() {
            let mut fim: usize = linha.len();
            while fim > inicio && (linha[fim - 1] == b'\n' || linha[fim - 1] == b'\r') {
                fim -= 1;
            }
            campos[indice_campo] = &linha[inicio..fim];
            indice_campo += 1;
        }

        if indice_campo != 7 {
            return Err(ErroCampo::CamposInsuficientes);
        }

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

/// Converte valor monetário com vírgula decimal (ex: "1000,50") em f64.
fn parse_capital(bytes: &[u8]) -> Result<f64, ErroCampo> {
    if bytes.is_empty() {
        return Ok(0.0);
    }
    let s: &str = str::from_utf8(bytes).map_err(|_| ErroCampo::NumeroInvalido)?;
    let s_formatado: String = s.replace(',', ".");
    s_formatado
        .parse::<f64>()
        .map_err(|_| ErroCampo::NumeroInvalido)
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
}
