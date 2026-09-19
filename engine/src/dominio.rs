//! Representação e validação tipada de tabelas de domínio/apoio da Receita Federal.

use crate::util::{ErroCampo, clean_quotes, parse_u16, parse_u32};
use std::str;

/// Registro validado e fortemente tipado de tabela de domínio/lookup.
#[derive(Debug, PartialEq, Eq)]
pub struct Dominio<'a, T> {
    pub codigo: T,
    pub descricao: &'a str,
}

#[inline]
fn split_two_fields(line: &[u8]) -> Result<(&[u8], &[u8]), ErroCampo> {
    let line = if line.ends_with(b"\r") {
        &line[..line.len() - 1]
    } else {
        line
    };
    let mut pos = None;
    for (i, &b) in line.iter().enumerate() {
        if b == b';' {
            pos = Some(i);
            break;
        }
    }
    let p = pos.ok_or(ErroCampo::CamposInsuficientes)?;
    Ok((&line[..p], &line[p + 1..]))
}

impl<'a> Dominio<'a, u32> {
    /// Faz o parse de linha para tabelas com chave u32 (ex.: Cnaes).
    ///
    /// # Parâmetros
    /// - `line`: Fatia de bytes de uma linha CSV.
    ///
    /// # Retorno
    /// - `Result<Self, ErroCampo>`: Registro tipado ou erro de validação.
    pub fn parse_line(line: &'a [u8]) -> Result<Self, ErroCampo> {
        let (raw_cod, raw_desc) = split_two_fields(line)?;
        let codigo = parse_u32(clean_quotes(raw_cod))?;
        let descricao =
            str::from_utf8(clean_quotes(raw_desc)).map_err(|_| ErroCampo::TextoInvalido)?;
        Ok(Self { codigo, descricao })
    }
}

impl<'a> Dominio<'a, u16> {
    /// Faz o parse de linha para tabelas com chave u16 (Motivos, Municipios, Naturezas, Paises, Qualificacoes).
    ///
    /// # Parâmetros
    /// - `line`: Fatia de bytes de uma linha CSV.
    ///
    /// # Retorno
    /// - `Result<Self, ErroCampo>`: Registro tipado ou erro de validação.
    pub fn parse_line(line: &'a [u8]) -> Result<Self, ErroCampo> {
        let (raw_cod, raw_desc) = split_two_fields(line)?;
        let codigo = parse_u16(clean_quotes(raw_cod))?;
        let descricao =
            str::from_utf8(clean_quotes(raw_desc)).map_err(|_| ErroCampo::TextoInvalido)?;
        Ok(Self { codigo, descricao })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_dominio_u32() {
        let line = b"\"6201501\";\"DESENVOLVIMENTO DE PROGRAMAS DE COMPUTADOR\"";
        let reg = Dominio::<u32>::parse_line(line).expect("Valid dominio u32");
        assert_eq!(reg.codigo, 6201501);
        assert_eq!(reg.descricao, "DESENVOLVIMENTO DE PROGRAMAS DE COMPUTADOR");
    }

    #[test]
    fn test_valid_dominio_u16() {
        let line = b"\"7107\";\"SAO PAULO\"";
        let reg = Dominio::<u16>::parse_line(line).expect("Valid dominio u16");
        assert_eq!(reg.codigo, 7107);
        assert_eq!(reg.descricao, "SAO PAULO");
    }

    #[test]
    fn test_dominio_missing_delimiter() {
        let line = b"\"7107\"";
        assert_eq!(
            Dominio::<u16>::parse_line(line),
            Err(ErroCampo::CamposInsuficientes)
        );
    }
}
