//! Representação e validação com tipagem forte dos registros da tabela Simples / MEI.

use crate::util::{ErroCampo, clean_quotes, opt_str, parse_opt_u32};

/// Registro validado e fortemente tipado da tabela Simples / MEI da Receita Federal.
#[derive(Debug, PartialEq)]
pub struct Simples<'a> {
    pub cnpj_basico: &'a [u8],
    pub opcao_simples: Option<&'a str>,
    pub data_opcao_simples: Option<u32>,
    pub data_exclusao_simples: Option<u32>,
    pub opcao_mei: Option<&'a str>,
    pub data_opcao_mei: Option<u32>,
    pub data_exclusao_mei: Option<u32>,
}

impl<'a> Simples<'a> {
    /// Faz o parse de uma fatia de bytes de uma linha CSV da tabela Simples.
    ///
    /// # Parâmetros
    /// - `line`: Fatia de bytes referente a uma linha única de texto CSV.
    ///
    /// # Retorno
    /// - `Result<Self, ErroCampo>`: Estrutura tipada ou erro de parsing/validação.
    pub fn parse_line(line: &'a [u8]) -> Result<Self, ErroCampo> {
        let mut campos = [&b""[..]; 7];
        let mut idx = 0;
        let mut inicio = 0;

        for (i, &b) in line.iter().enumerate() {
            if b == b';' {
                if idx < 7 {
                    campos[idx] = &line[inicio..i];
                    idx += 1;
                    inicio = i + 1;
                } else {
                    break;
                }
            }
        }

        if idx < 7 && inicio <= line.len() {
            campos[idx] = &line[inicio..];
            idx += 1;
        }

        if idx < 7 {
            return Err(ErroCampo::CamposInsuficientes);
        }

        let cnpj_basico = clean_quotes(campos[0]);
        if cnpj_basico.len() != 8 || !cnpj_basico.iter().all(|b| b.is_ascii_alphanumeric()) {
            return Err(ErroCampo::CnpjInvalido);
        }

        let opcao_simples = opt_str(clean_quotes(campos[1]))?;
        let data_opcao_simples = parse_opt_u32(clean_quotes(campos[2]))?;
        let data_exclusao_simples = parse_opt_u32(clean_quotes(campos[3]))?;
        let opcao_mei = opt_str(clean_quotes(campos[4]))?;
        let data_opcao_mei = parse_opt_u32(clean_quotes(campos[5]))?;
        let data_exclusao_mei = parse_opt_u32(clean_quotes(campos[6]))?;

        Ok(Self {
            cnpj_basico,
            opcao_simples,
            data_opcao_simples,
            data_exclusao_simples,
            opcao_mei,
            data_opcao_mei,
            data_exclusao_mei,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_simples_line() {
        let line = b"\"12345678\";\"S\";\"20200101\";\"20211231\";\"N\";\"\";\"\"";
        let reg = Simples::parse_line(line).expect("Valid simples line");
        assert_eq!(reg.cnpj_basico, b"12345678");
        assert_eq!(reg.opcao_simples, Some("S"));
        assert_eq!(reg.data_opcao_simples, Some(20200101));
        assert_eq!(reg.data_exclusao_simples, Some(20211231));
        assert_eq!(reg.opcao_mei, Some("N"));
        assert_eq!(reg.data_opcao_mei, None);
        assert_eq!(reg.data_exclusao_mei, None);
    }

    #[test]
    fn test_simples_insufficient_fields() {
        let line = b"\"12345678\";\"S\"";
        assert_eq!(
            Simples::parse_line(line),
            Err(ErroCampo::CamposInsuficientes)
        );
    }

    #[test]
    fn test_simples_invalid_cnpj() {
        let line = b"\"1234\";\"S\";\"\";\"\";\"\";\"\";\"\"";
        assert_eq!(Simples::parse_line(line), Err(ErroCampo::CnpjInvalido));
    }
}
