//! Utilitários de parsing de campos e remoção de delimitadores com zero-copy.

use std::str;

/// Erros estritos de validação e parsing de campos.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ErroCampo {
    CnpjInvalido,
    NumeroInvalido,
    TextoInvalido,
    CamposInsuficientes,
}

/// Remove aspas delimitadoras externas de um campo, se presentes.
#[inline]
pub fn clean_quotes(campo: &[u8]) -> &[u8] {
    if campo.len() >= 2 && campo.first() == Some(&b'"') && campo.last() == Some(&b'"') {
        &campo[1..campo.len() - 1]
    } else {
        campo
    }
}

/// Acumula dígitos ASCII em u64 com detecção de overflow (núcleo único compartilhado).
#[inline]
fn parse_ascii_digits(bytes: &[u8]) -> Result<u64, ErroCampo> {
    if bytes.is_empty() {
        return Ok(0);
    }
    let mut val: u64 = 0;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return Err(ErroCampo::NumeroInvalido);
        }
        val = val
            .checked_mul(10)
            .and_then(|v| v.checked_add((b - b'0') as u64))
            .ok_or(ErroCampo::NumeroInvalido)?;
    }
    Ok(val)
}

/// Converte fatia de bytes de dígitos ASCII em u8.
#[inline]
pub fn parse_u8(bytes: &[u8]) -> Result<u8, ErroCampo> {
    u8::try_from(parse_ascii_digits(bytes)?).map_err(|_| ErroCampo::NumeroInvalido)
}

/// Converte fatia de bytes de dígitos ASCII em u16.
#[inline]
pub fn parse_u16(bytes: &[u8]) -> Result<u16, ErroCampo> {
    u16::try_from(parse_ascii_digits(bytes)?).map_err(|_| ErroCampo::NumeroInvalido)
}

/// Converte fatia de bytes de dígitos ASCII em u32.
#[inline]
pub fn parse_u32(bytes: &[u8]) -> Result<u32, ErroCampo> {
    u32::try_from(parse_ascii_digits(bytes)?).map_err(|_| ErroCampo::NumeroInvalido)
}

/// Converte fatia de bytes opcional em Option<u16>.
#[inline]
pub fn parse_opt_u16(bytes: &[u8]) -> Result<Option<u16>, ErroCampo> {
    if bytes.is_empty() {
        Ok(None)
    } else {
        parse_u16(bytes).map(Some)
    }
}

/// Converte fatia de bytes opcional em Option<u32>.
#[inline]
pub fn parse_opt_u32(bytes: &[u8]) -> Result<Option<u32>, ErroCampo> {
    if bytes.is_empty() {
        Ok(None)
    } else {
        parse_u32(bytes).map(Some)
    }
}

/// Converte fatia de bytes opcional em Option<&str>.
#[inline]
pub fn opt_str(bytes: &[u8]) -> Result<Option<&str>, ErroCampo> {
    if bytes.is_empty() {
        Ok(None)
    } else {
        str::from_utf8(bytes)
            .map(Some)
            .map_err(|_| ErroCampo::TextoInvalido)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_u8_valid_and_overflow() {
        assert_eq!(parse_u8(b"0").unwrap(), 0);
        assert_eq!(parse_u8(b"255").unwrap(), 255);
        assert_eq!(parse_u8(b"256").unwrap_err(), ErroCampo::NumeroInvalido);
        assert_eq!(parse_u8(b"").unwrap(), 0);
        assert_eq!(parse_u8(b"abc").unwrap_err(), ErroCampo::NumeroInvalido);
    }

    #[test]
    fn test_parse_u16_valid_and_overflow() {
        assert_eq!(parse_u16(b"65535").unwrap(), 65535);
        assert_eq!(parse_u16(b"65536").unwrap_err(), ErroCampo::NumeroInvalido);
    }

    #[test]
    fn test_parse_u32_valid() {
        assert_eq!(parse_u32(b"12345678").unwrap(), 12345678);
    }

    #[test]
    fn test_parse_opt_helpers() {
        assert_eq!(parse_opt_u16(b"").unwrap(), None);
        assert_eq!(parse_opt_u16(b"123").unwrap(), Some(123));
        assert_eq!(parse_opt_u32(b"").unwrap(), None);
        assert_eq!(parse_opt_u32(b"987654").unwrap(), Some(987654));
    }
}
