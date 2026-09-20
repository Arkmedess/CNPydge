# Motor Nativo CNPydge (`engine`)

Extensão CPython de alta vazão compilada em Rust para leitura, parsing e conversão colunar dos dados públicos de CNPJ da Receita Federal em Apache Parquet.

## 1. Propósito
O módulo `engine` resolve o gargalo de I/O e processamento de arquivos massivos (>50M registros) da Receita Federal. Ele executa a leitura mapeada em memória (`mmap`), particionamento paralelo com Rayon e serialização direta em Apache Parquet liberando integralmente o GIL do Python.

## 2. Contrato Público (Bindings PyO3)

A crate expõe a biblioteca dinâmica `cnpydge._core` com as seguintes funções públicas:

- `version() -> str`: Retorna a versão da crate compilada (`CARGO_PKG_VERSION`).
- `to_parquet(src: str, dst: str, kind: str | None = None) -> int`:
  - **Parâmetros:**
    - `src`: Caminho absoluto ou relativo para o arquivo CSV de entrada.
    - `dst`: Caminho de destino para o arquivo `.parquet` colunar.
    - `kind`: Identificador da tabela (`empresas`, `socios`, `estabelecimentos`, `simples`, `cnaes`, etc.). Padrão: `"empresas"`.
  - **Retorno:** Número total de registros processados e gravados (`usize`).
  - **Erros:** Lança `PyRuntimeError` em caso de falha de I/O, formato corrompido, erro de esquema Arrow ou tipo de tabela não suportado.
  - **Concorrência:** Executa inteiramente sob `py.allow_threads`, liberando o GIL.

## 3. Módulos Internos
- **`reader.rs`:** `MmapReader` gerencia o mapeamento de memória virtual via `memmap2` e cálculo de fatias contíguas alinhadas a quebras de linha (`\n`).
- **`converter.rs`:** Orquestrador que distribui blocos pelo pool Rayon, instancia construtores Arrow e escreve em Parquet (Snappy) via `ArrowWriter`.
- **Parsers de Domínio (`empresa.rs`, `socio.rs`, `estabelecimento.rs`, `simples.rs`, `dominio.rs`):** Fatiamento zero-copy de bytes, remoção de aspas e conversão estrita de tipos.
- **`util.rs`:** Utilitários de conversão numérica (`parse_u8`, `parse_u16`, `parse_u32`), monetária (`parse_capital`) e limpeza de aspas (`clean_quotes`).

## 4. Invariantes de Domínio
1. **Zero-Copy em Limites de Bloco:** A leitura de arquivos mapeados não aloca strings temporárias para limites de bloco ou linhas brutas.
2. **Suporte Alfanumérico:** Toda validação de CNPJ aceita caracteres alfanuméricos ASCII (`[0-9A-Za-z]`).
3. **Datas como Primitivos Numéricos:** Todas as datas `YYYYMMDD` válidas são coerzidas para `UInt32` no Parquet.

## 5. Testes, Linters e Benchmarks
- **Testes Unitários:** `cargo test`
- **Análise Estática & Linter:** `cargo clippy --all-targets`
- **Harness de Benchmarks:** `cargo bench` (executa benchmarks Criterion em `benches/parser_bench.rs`)