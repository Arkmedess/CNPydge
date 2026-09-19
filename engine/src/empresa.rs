//! Representação e validação com tipagem forte dos registros da tabela de Empresas do CNPJ.

use std::str;

/// Erros estritos de validação e parsing de campos de Empresa.
#[derive(Debug, PartialEq, Eq)]
pub enum ErroCampo {
    CnpjInvalido,
    NumeroInvalido,
    TextoInvalido,
    CamposInsuficientes,
}

/// Registro validado e fortemente tipado de uma Empresa.
///
/// Mantém termos canônicos da Receita Federal sem tradução.
/// A referência de tempo de vida `'a` vincula os campos diretamente à memória
/// mapeada (`mmap`), garantindo zero alocações na heap durante o parsing.
#[derive(Debug, PartialEq)]
pub struct Empresa<'a> {
    pub cnpj: &'a [u8],
    pub razao: &'a str,
    pub nat_jur: u16,
    pub qualif: u16,
    pub capital: f64,
    pub porte: u8,
    pub ente_fed: Option<&'a str>,
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

        let nat_jur_raw: &'a [u8] = clean_quotes(campos[2]);
        let nat_jur: u16 = parse_u16(nat_jur_raw)?;

        let qualif_raw: &'a [u8] = clean_quotes(campos[3]);
        let qualif: u16 = parse_u16(qualif_raw)?;

        let capital_raw: &'a [u8] = clean_quotes(campos[4]);
        let capital: f64 = parse_capital(capital_raw)?;

        let porte_raw: &'a [u8] = clean_quotes(campos[5]);
        let porte: u8 = parse_u8(porte_raw)?;

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

/// Remove aspas delimitadoras externas de um campo, se presentes.
#[inline]
fn clean_quotes(campo: &[u8]) -> &[u8] {
    if campo.len() >= 2 && campo.first() == Some(&b'"') && campo.last() == Some(&b'"') {
        &campo[1..campo.len() - 1]
    } else {
        campo
    }
}

/// Converte fatia de bytes de dígitos ASCII em u16 com tipagem estrita.
#[inline]
fn parse_u16(bytes: &[u8]) -> Result<u16, ErroCampo> {
    if bytes.is_empty() {
        return Ok(0);
    }
    let mut valor: u16 = 0;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return Err(ErroCampo::NumeroInvalido);
        }
        valor = valor
            .checked_mul(10)
            .and_then(|v: u16| v.checked_add((b - b'0') as u16))
            .ok_or(ErroCampo::NumeroInvalido)?;
    }
    Ok(valor)
}

/// Converte fatia de bytes de dígitos ASCII em u8 com tipagem estrita.
#[inline]
fn parse_u8(bytes: &[u8]) -> Result<u8, ErroCampo> {
    if bytes.is_empty() {
        return Ok(0);
    }
    let mut valor: u8 = 0;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return Err(ErroCampo::NumeroInvalido);
        }
        valor = valor
            .checked_mul(10)
            .and_then(|v: u8| v.checked_add(b - b'0'))
            .ok_or(ErroCampo::NumeroInvalido)?;
    }
    Ok(valor)
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
