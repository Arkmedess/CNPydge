# ADR 0001: Python + Rust via PyO3/maturin

## Status
Aceito

## Contexto
O projeto precisa importar a base publica de CNPJ da Receita Federal -- centenas de milhoes de registros, centenas de GB. A decisao fundamental e: em que linguagem rodar o hot path (parsing, normalizacao, batching, persistencia) e como integrar com a camada de orquestracao.

## Decisao
- **Hot path em Rust**: parsing CSV, normalizacao zero-copy, pipeline lock-free, batch builder e COPY BINARY sao implementados em `rust/cnpj_core`.
- **Orquestracao em Python 3.14**: CLI (Typer), config (Pydantic), download (httpx), dashboard (Rich), API (FastAPI), logs (loguru).
- **Integracao via PyO3 + maturin**: o nucleo Rust e exposto como modulo Python via PyO3, empacotado com maturin para build distribuivel.

## Consequencias
- Performance de parsing e processamento limitada pelo hardware, nao pela linguagem.
- Fronteira Python/Rust bem definida: se itera sobre registros do CSV, e Rust; se orquestra, e Python.
- Build requer toolchain dupla (Python + Rust), mas maturin automatiza o processo.
- Debugging do hot path e mais complexo (Rust), mas o numero de bugs criticos cai drasticamente.

## Alternativas consideradas
- **Python puro (pandas/polars)**: descartado por nao atingir throughput necessario com memoria constante. DataFrames eager violam o principio de streaming.
- **Python + Cython**: descartado por complexidade de build e menor maturidade do ecossistema vs Rust.
- **Go + cgo**: descartado por ecossistema de parsing CSV menos maduro e integracao com Python menos fluida.
