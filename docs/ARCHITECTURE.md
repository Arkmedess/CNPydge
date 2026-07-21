# Arquitetura do CNPydge

> Documento vivo. Atualizado conforme decisoes sao tomadas e estrutura evolui.
> Ver `docs/adr/` para decisoes de arquitetura detalhadas.

## Visao geral

O CNPydge separa responsabilidades em dois dominios:

- **Python (orquestracao)**: CLI, config, download, dashboard, API, logs.
- **Rust (hot path)**: mmap, parser SIMD, normalizacao zero-copy, pipeline lock-free, batch builder, COPY BINARY.

Integracao via **PyO3 + maturin** (ver ADR-0001).

## Estrutura alvo

```text
CNPydge/
├── rust/
│   └── cnpj_core/           # hot path: mmap, parser, normalizacao, pipeline, batch, destinos
├── python/
│   ├── app/                 # inicializacao e composicao dos servicos
│   ├── features/            # funcionalidades por dominio (Vertical Slice)
│   │   ├── updater/         # download e atualizacao da base RFB
│   │   ├── company/         # consulta e manipulacao de CNPJ
│   │   ├── search/          # mecanismos de pesquisa
│   │   ├── export/          # exportacao (CSV, Parquet, DuckDB, Postgres)
│   │   ├── metrics/         # metricas e monitoramento
│   │   └── benchmark/       # benchmarks de desempenho
│   ├── shared/              # codigo compartilhado entre features
│   ├── integrations/        # adaptadores para servicos externos
│   │   ├── duckdb/
│   │   ├── postgres/
│   │   ├── receita/
│   │   └── storage/
│   ├── config/              # configuracao central (settings, logging, paths)
│   ├── entrypoints/         # interfaces de entrada (API, CLI, Dashboard)
│   └── tests/               # testes unitarios e integracao
├── docs/
│   ├── adr/                 # Architecture Decision Records
│   ├── formato-rfb/         # notas sobre leiaute dos arquivos da RFB
│   └── benchmarks/          # resultados historicos de benchmarks
├── benchmarks/              # datasets e scripts de benchmark
├── docker/
├── scripts/
└── .mimocode/               # skills e agentes do MiMo Code
```

## Estrutura atual (com legado)

O repositorio contem codigo legado que sera migrado gradualmente:

| Diretorio legado | Destino na arquitetura nova | Status |
|---|---|---|
| `python/api/` | `python/entrypoints/api.py` | Legado -- migrar |
| `python/cli/` | `python/entrypoints/cli.py` | Legado -- migrar |
| `python/dashboard/` | `python/entrypoints/dashboard.py` | Legado -- migrar |
| `python/updater/` | `python/features/updater/` | Legado -- migrar |
| `python/benchmark/` | `python/features/benchmark/` | Legado -- migrar |

**Regra**: nao adicionar funcionalidade nova nos diretorios legados. Novas features vao para a estrutura alvo.

## Principios de organizacao

1. Python cuida da orquestracao, da interface externa e do controle de fluxo.
2. Rust cuida do hot path, do parsing e da transformacao de alto volume.
3. Cada feature e um slice vertical, com limites claros entre dominio e infraestrutura.
4. Novos destinos de dados entram como adaptadores (traits), nao como condicionais hardcoded.
5. Toda decisao de arquitetura e documentada como ADR em `docs/adr/`.

## Fronteira Python <-> Rust

```
Python (orquestracao)              Rust (cnpj_core via PyO3)
--------------------------------  ------------------------------------
CLI (Typer)                        mmap (memmap2)
Config (Pydantic)                  Parser CSV SIMD (csv-core/memchr/simdutf8)
Download (httpx)                   Normalizacao zero-copy
Dashboard (Rich)                   Pipeline lock-free (crossbeam)
API (FastAPI)                      Batch builder (100k registros)
Logs (loguru)                      COPY BINARY (Postgres) / DuckDB / Arrow
```

Regra de bolso: **se envolve iterar sobre registros do CSV, e Rust.** Se envolve orquestrar, e Python.

## Roadmap tecnico

Ver `.mimocode/skills/core/planning/SKILL.md` para o roadmap completo (Fase 1 a 9).

## ADRs

| ADR | Titulo | Status |
|---|---|---|
| [0001](adr/0001-python-rust-via-pyo3.md) | Python + Rust via PyO3/maturin | Aceito |
| [0002](adr/0002-mmap-vs-buffered-read.md) | mmap vs Buffered Read | Aceito |
| [0003](adr/0003-crossbeam-vs-tokio-para-pipeline.md) | crossbeam vs tokio para pipeline | Aceito |
| [0004](adr/0004-vertical-slice-architecture.md) | Vertical Slice Architecture | Aceito |
| [0005](adr/0005-formato-cnpj-alfanumerico.md) | Formato CNPJ alfanumerico | Aceito |
| [0006](adr/0006-plugin-system-para-destinos.md) | Plugin system para destinos | Proposto (Fase 9) |
| [0007](adr/0007-copy-binary-para-postgres.md) | COPY BINARY para Postgres | Aceito |
