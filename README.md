# CNPydge

Pipeline de ingestão de alta performance da base pública de CNPJ da Receita Federal.
Python 3.14 (orquestração) + Rust (hot path via PyO3/maturin).

> O limite deve ser o hardware, nunca o software.

## Visão

Importar centenas de milhões de registros da base pública do CNPJ com throughput limitado
pelo disco (SSD NVMe: ~3 GB/s), não pela linguagem. Memória praticamente constante
independente do tamanho do arquivo.

Veja `docs/PHILOSOPHY.md` e `docs/VISION.md` para o manifesto completo.

## Stack

| Camada | Tecnologia | Responsabilidade |
|---|---|---|
| Hot path | Rust (`cnpj_core`) | mmap, parser SIMD, normalização zero-copy, pipeline lock-free, batch, COPY BINARY |
| Orquestração | Python 3.14 | CLI (Typer), config (Pydantic), download (httpx), dashboard (Rich), API (FastAPI) |
| Integração | PyO3 + maturin | Fronteira Python/Rust com zero-copy |

## Estrutura

```text
CNPydge/
├── rust/cnpj_core/       # hot path: mmap, parser SIMD, normalização, pipeline, batch, destinos
├── python/
│   ├── app/               # inicialização e composição dos serviços
│   ├── features/          # funcionalidades por domínio (Vertical Slice)
│   │   ├── updater/       # download e atualização da base RFB
│   │   ├── company/       # consulta e manipulação de CNPJ
│   │   ├── search/        # mecanismos de pesquisa
│   │   ├── export/        # exportação (CSV, Parquet, DuckDB, Postgres)
│   │   ├── metrics/       # métricas e monitoramento
│   │   └── benchmark/     # benchmarks de desempenho
│   ├── shared/            # código compartilhado entre features
│   ├── integrations/      # adaptadores para serviços externos
│   │   ├── duckdb/
│   │   ├── postgres/
│   │   ├── receita/
│   │   └── storage/
│   ├── config/            # configuração central (settings, logging, paths)
│   ├── entrypoints/       # interfaces de entrada (API, CLI, Dashboard)
│   └── tests/             # testes unitários e integração
├── docs/
│   ├── adr/               # Architecture Decision Records
│   ├── formato-rfb/       # notas sobre leiaute dos arquivos da RFB
│   └── benchmarks/        # resultados históricos de benchmarks
├── benchmarks/            # datasets e scripts de benchmark
├── docker/
├── scripts/
└── .mimocode/             # skills e agentes do MiMo Code
```

## Setup rápido

```bash
./scripts/bootstrap.sh
```

Requer [uv](https://docs.astral.sh/uv/) e [Rust](https://rustup.rs/). O script sincroniza
dependências com `uv sync`, builda a extensão Rust via maturin e roda os testes.

## Uso rápido

### Importar base completa

```bash
# Configurar conexão com o banco
export CNPJ_POSTGRES_DSN="postgresql://localhost:5432/cnpj"

# Importar
uv run python -m python.entrypoints.cli importar
```

### Importar arquivos específicos

```bash
uv run python -m python.entrypoints.cli importar \
  --files Empresas0.zip Estabelecimentos0.zip
```

### Ver status do cache

```bash
uv run python -m python.entrypoints.cli status
```

### Uso via Python

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

## Roadmap

| Fase | Descrição | Status |
|---|---|---|
| 1 | Pipeline mínimo (download -> parser -> COPY Postgres) | ✅ Concluído |
| 2 | SIMD (memchr/csv-core/simdutf8) | Pendente |
| 3 | Apache Arrow (RecordBatch intermediário) | Pendente |
| 4 | DuckDB (destino de analytics) | Pendente |
| 5 | Atualização incremental (diffs mensais RFB) | Pendente |
| 6 | Dashboard (observabilidade visual) | Pendente |
| 7 | API REST (FastAPI) | Pendente |
| 8 | Benchmark automatizado (Criterion + CI) | Pendente |
| 9 | Plugin system (ClickHouse, Elastic, SQLite, Parquet, S3, BigQuery) | Pendente |

Ver `.mimocode/skills/core/planning/SKILL.md` para detalhes de cada fase.

## Documentação

### Guia de desenvolvimento

- `docs/DEVELOPMENT.md` -- Guia completo para desenvolvedores
- `CONTRIBUTING.md` -- Diretrizes de contribuição
- `docs/USE_CASES.md` -- Casos de uso práticos

### Arquitetura e decisões

- `docs/ARCHITECTURE.md` -- Visão arquitetural e estrutura alvo vs atual
- `docs/adr/` -- Architecture Decision Records (por que cada decisão foi tomada)
- `docs/PHILOSOPHY.md` -- Manifesto e princípios de engenharia
- `docs/VISION.md` -- Visão do projeto e stack tecnológica

### Módulos

- `rust/cnpj_core/README.md` -- Documentação do módulo Rust (hot path)
- `python/README.md` -- Documentação da camada Python (orquestração)

### Formato dos dados

- `docs/formato-rfb/README.md` -- Layout dos arquivos da Receita Federal

## Performance

| Métrica | Valor |
|---|---|
| Throughput | ~1.4M registros/s (SSD NVMe) |
| RAM | ~50MB constante |
| Tempo (base completa) | ~3 minutos |
| Arquivos processados | 36 ZIPs (~50GB) |

Ver `docs/benchmarks/README.md` para resultados históricos.

## Licença

Proprietary -- ver `LICENSE` para detalhes.
