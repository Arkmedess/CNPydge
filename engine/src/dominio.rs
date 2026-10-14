//! Representação e validação tipada de tabelas de domínio/apoio da Receita Federal.

use crate::converter::{ConvertErr, TableDescriptor};
use crate::util::{ErroCampo, clean_quotes, parse_u16, parse_u32};
use arrow::array::{ArrayRef, StringBuilder, UInt16Builder, UInt32Builder};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;
use std::marker::PhantomData;
use std::str;
use std::sync::Arc;

/// Registro validado e fortemente tipado de tabela de domínio/lookup.
#[derive(Debug, PartialEq, Eq)]
pub struct Dominio<'a, T> {
    pub codigo: T,
    pub descricao: &'a str,
}

#[inline]
fn split_two_fields(line: &[u8]) -> Result<(&[u8], &[u8]), ErroCampo> {
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    let p = line
        .iter()
        .position(|&b| b == b';')
        .ok_or(ErroCampo::CamposInsuficientes)?;
    Ok((&line[..p], &line[p + 1..]))
}

impl<'a> Dominio<'a, u32> {
    /// Faz o parse de linha para tabelas com chave u32 (ex.: Cnaes).
    ///
    /// ### Parâmetros
    /// - `line`: Fatia de bytes de uma linha CSV.
    ///
    /// ### Retorno
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
    /// ### Parâmetros
    /// - `line`: Fatia de bytes de uma linha CSV.
    ///
    /// ### Retorno
    /// - `Result<Self, ErroCampo>`: Registro tipado ou erro de validação.
    pub fn parse_line(line: &'a [u8]) -> Result<Self, ErroCampo> {
        let (raw_cod, raw_desc) = split_two_fields(line)?;
        let codigo = parse_u16(clean_quotes(raw_cod))?;
        let descricao =
            str::from_utf8(clean_quotes(raw_desc)).map_err(|_| ErroCampo::TextoInvalido)?;
        Ok(Self { codigo, descricao })
    }
}

/// Descritor genérico para tabelas de domínio/lookup da RFB com chave numérica tipada (u16 ou u32).
#[derive(Debug, Clone, Copy)]
pub struct DominioDescriptor<K> {
    name: &'static str,
    _marker: PhantomData<K>,
}

impl<K> DominioDescriptor<K> {
    /// Cria uma nova instância de descritor com nome canônico.
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }
}

pub const CNAES_DESCRIPTOR: DominioDescriptor<u32> = DominioDescriptor::new("cnaes");
pub const MOTIVOS_DESCRIPTOR: DominioDescriptor<u16> = DominioDescriptor::new("motivos");
pub const MUNICIPIOS_DESCRIPTOR: DominioDescriptor<u16> = DominioDescriptor::new("municipios");
pub const NATUREZAS_DESCRIPTOR: DominioDescriptor<u16> = DominioDescriptor::new("naturezas");
pub const PAISES_DESCRIPTOR: DominioDescriptor<u16> = DominioDescriptor::new("paises");
pub const QUALIFICACOES_DESCRIPTOR: DominioDescriptor<u16> =
    DominioDescriptor::new("qualificacoes");

impl TableDescriptor for DominioDescriptor<u32> {
    fn name(&self) -> &'static str {
        self.name
    }

    fn schema(&self) -> SchemaRef {
        Arc::new(Schema::new(vec![
            Field::new("codigo", DataType::UInt32, false),
            Field::new("descricao", DataType::Utf8, false),
        ]))
    }

    fn build_batch(&self, slice: &[u8]) -> Result<Option<RecordBatch>, ConvertErr> {
        if slice.is_empty() {
            return Ok(None);
        }

        let mut cod_b = UInt32Builder::new();
        let mut desc_b = StringBuilder::new();
        let mut row_count = 0;

        for line in slice.split(|&b| b == b'\n') {
            let line = line.strip_suffix(b"\r").unwrap_or(line);
            if line.is_empty() {
                continue;
            }

            let Ok(dom) = Dominio::<u32>::parse_line(line) else {
                continue;
            };
            cod_b.append_value(dom.codigo);
            desc_b.append_value(dom.descricao);
            row_count += 1;
        }

        if row_count == 0 {
            Ok(None)
        } else {
            let columns: Vec<ArrayRef> = vec![Arc::new(cod_b.finish()), Arc::new(desc_b.finish())];
            Ok(Some(RecordBatch::try_new(self.schema(), columns)?))
        }
    }
}

impl TableDescriptor for DominioDescriptor<u16> {
    fn name(&self) -> &'static str {
        self.name
    }

    fn schema(&self) -> SchemaRef {
        Arc::new(Schema::new(vec![
            Field::new("codigo", DataType::UInt16, false),
            Field::new("descricao", DataType::Utf8, false),
        ]))
    }

    fn build_batch(&self, slice: &[u8]) -> Result<Option<RecordBatch>, ConvertErr> {
        if slice.is_empty() {
            return Ok(None);
        }

        let mut cod_b = UInt16Builder::new();
        let mut desc_b = StringBuilder::new();
        let mut row_count = 0;

        for line in slice.split(|&b| b == b'\n') {
            let line = line.strip_suffix(b"\r").unwrap_or(line);
            if line.is_empty() {
                continue;
            }

            let Ok(dom) = Dominio::<u16>::parse_line(line) else {
                continue;
            };
            cod_b.append_value(dom.codigo);
            desc_b.append_value(dom.descricao);
            row_count += 1;
        }

        if row_count == 0 {
            Ok(None)
        } else {
            let columns: Vec<ArrayRef> = vec![Arc::new(cod_b.finish()), Arc::new(desc_b.finish())];
            Ok(Some(RecordBatch::try_new(self.schema(), columns)?))
        }
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

    #[test]
    fn test_dominio_descriptor_u32_batch() {
        let csv = b"\"6201501\";\"DESENVOLVIMENTO DE PROGRAMAS\"\n";
        let batch = CNAES_DESCRIPTOR
            .build_batch(csv)
            .expect("Sucesso batch cnaes")
            .expect("RecordBatch não nulo");
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 2);
    }

    #[test]
    fn test_dominio_descriptor_u16_batch() {
        let csv = b"\"7107\";\"SAO PAULO\"\n";
        let batch = MUNICIPIOS_DESCRIPTOR
            .build_batch(csv)
            .expect("Sucesso batch municipios")
            .expect("RecordBatch não nulo");
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 2);
    }
}
