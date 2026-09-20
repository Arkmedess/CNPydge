//! Motor nativo compilado em Rust com bindings PyO3 para processamento analítico do CNPJ.
//!
//! Fornece bindings de alta vazão para descarregar o parsing e a conversão de arquivos
//! massivos do CNPJ diretamente para Apache Parquet com liberação explícita do GIL.

#![allow(unsafe_op_in_unsafe_fn, clippy::useless_conversion)]

pub mod converter;
pub mod dominio;
pub mod empresa;
pub mod estabelecimento;
pub mod reader;
pub mod simples;
pub mod socio;
pub mod util;

use converter::TableKind;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

/// Retorna a versão semântica do motor nativo compilado em Rust.
///
/// ### Retorno
/// Retorna a versão estática declarada no `Cargo.toml` (`CARGO_PKG_VERSION`).
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Converte um arquivo CSV do CNPJ para Apache Parquet liberando o GIL do Python.
///
/// ### Parâmetros
/// - `py`: Marcador de thread do runtime Python utilizado para manipulação do GIL.
/// - `src`: Caminho para o arquivo CSV de origem no disco.
/// - `dst`: Caminho de destino onde o arquivo Parquet colunar será criado.
/// - `kind`: Nome opcional da entidade cadastral do CNPJ (padrão: `"empresas"`).
///
/// ### Retorno
/// Retorna `PyResult<usize>` contendo o número total de linhas processadas e persistidas.
///
/// ### Erros
/// Lança `PyRuntimeError` caso o arquivo fonte não possa ser lido, o tipo da tabela
/// não seja reconhecido ou ocorra falha de serialização nos batches Arrow/Parquet.
///
/// ### Concorrência
/// Toda a operação de leitura mapeada, parsing e escrita colunar é executada sob
/// `py.allow_threads`, liberando totalmente o GIL do interpretador Python.
#[pyfunction]
#[pyo3(signature = (src, dst, kind=None))]
fn to_parquet(py: Python<'_>, src: &str, dst: &str, kind: Option<&str>) -> PyResult<usize> {
    let table_kind: TableKind = match kind {
        Some(k) => TableKind::parse(k).map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        None => TableKind::Empresas,
    };

    py.allow_threads(|| {
        converter::to_parquet(src, dst, table_kind)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    })
}

/// Ponto de entrada e registro de funções do módulo nativo CPython (`cnpydge._core`).
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(to_parquet, m)?)?;
    Ok(())
}
