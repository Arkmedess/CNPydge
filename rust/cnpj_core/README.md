# cnpj_core

Núcleo de alta performance em Rust para processamento de dados de CNPJ da Receita Federal.

## Responsabilidade

Este módulo implementa o **hot path** do pipeline:

- **mmap**: Acesso a arquivos CSV via memory-mapping (zero-copy)
- **parser**: Parsing CSV vetORIZADO (SIMD) para delimitador `;`, encoding Latin-1
- **normalize**: Normalização de registros (concatenação CNPJ, validação)
- **pipeline**: Pipeline paralelo com crossbeam channels (4 threads)
- **batch**: Acumulação de lotes (100k registros padrão)
- **postgres**: Escrita via protocolo COPY BINARY do PostgreSQL
- **metrics**: Coleta de métricas de performance

## Arquitetura

```
Reader Thread → Parser Thread → Batcher Thread → Writer Thread
     ↓              ↓               ↓               ↓
   mmap         parse+normalize   batch 100k     COPY BINARY
```

Cada thread se comunica via `crossbeam-channel` bounded (capacidade 8), proporcionando backpressure natural.

## Uso via Python (PyO3)

```python
import cnpj_core

stats = cnpj_core.importar(
    files=["Empresas0.csv", "Estabelecimentos0.csv"],
    dsn="postgresql://localhost:5432/cnpj",
    batch_size=100_000,
)

print(f"Registros: {stats.records_processed}")
print(f"Throughput: {stats.records_per_sec:.0f} registros/s")
```

## Compilação

```bash
# Desenvolvimento
uv run maturin develop

# Release (otimizado)
uv run maturin develop --release
```

## Testes

```bash
# Unitários
cargo test

# Benchmarks
cargo bench
```

## Estrutura dos módulos

| Módulo | Responsabilidade |
|---|---|
| `lib.rs` | Interface PyO3, exportação para Python |
| `mmap.rs` | Memory-mapping de arquivos CSV |
| `parser.rs` | Parsing CSV com memchr/csv-core |
| `normalize.rs` | Normalização de campos, concatenação CNPJ |
| `channels.rs` | Tipos de dados para crossbeam channels |
| `batch.rs` | Acumulação de lotes com reciclagem de Vec |
| `pipeline.rs` | Orquestração do pipeline 4-thread |
| `postgres.rs` | Escrita COPY BINARY no PostgreSQL |
| `metrics.rs` | Coleta de throughput, latência, erros |
| `simd.rs` | Operações SIMD para busca de delimitador |
| `filters.rs` | Filtros de registros (preparado para Fase 9) |
| `arrow.rs` | Apache Arrow RecordBatch (preparado para Fase 3) |
| `duckdb.rs` | Integração DuckDB (preparado para Fase 4) |

## Princípios de engenharia

1. **Zero-copy**: Dados lidos diretamente do mmap, sem cópias desnecessárias
2. **Streaming**: Processamento registro a registro, sem carregar arquivo inteiro em memória
3. **Memória constante**: Uso de RAM independente do tamanho do arquivo
4. **Sem panic no hot path**: Todos os erros retornam `Result`, nunca `unwrap()`
5. **GIL release**: `py.allow_threads()` durante toda execução do pipeline

## Formato dos dados de entrada

- **Delimitador**: `;` (ponto e vírgula)
- **Encoding**: ISO-8859-1 (Latin-1)
- **Quebra de linha**: `\r\n` (CRLF)
- **CNPJ**: Alfanumérico (14 caracteres: 12 + 2 DV)

Ver `docs/formato-rfb/README.md` para detalhes completos.

## Dependências principais

| Crate | Versão | Uso |
|---|---|---|
| `pyo3` | 0.23 | Interface Python/Rust |
| `memmap2` | 0.9 | Memory-mapping |
| `crossbeam` | 0.8 | Canais paralelos |
| `memchr` | 2.7 | Busca SIMD de delimitador |
| `csv-core` | 0.1 | Parsing CSV de baixo nível |
| `simdutf8` | 0.1 | Validação UTF-8 SIMD |
| `thiserror` | 2.0 | Tratamento de erros |
| `postgres` | 0.19 | Cliente PostgreSQL |
| `tracing` | 0.1 | Instrumentação |

## Licença

Proprietary -- ver `LICENSE` para detalhes.
