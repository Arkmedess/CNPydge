# CNPydge

Pipeline de ingestao de alta performance da base publica de CNPJ da Receita Federal.
Python 3.14 (orquestracao) + Rust (hot path via PyO3/maturin).

> O limite deve ser o hardware, nunca o software.

## Visao

Importar centenas de milhoes de registros da base publica do CNPJ com throughput limitado
pelo disco (SSD NVMe: ~3 GB/s), nao pela linguagem. Memoria praticamente constante
independente do tamanho do arquivo.

Veja `docs/PHILOSOPHY.md` e `docs/VISION.md` para o manifesto completo.

## Stack

| Camada | Tecnologia | Responsabilidade |
|---|---|---|
| Hot path | Rust (`cnpj_core`) | mmap, parser SIMD, normalizacao zero-copy, pipeline lock-free, batch, COPY BINARY |
| Orquestracao | Python 3.14 | CLI (Typer), config (Pydantic), download (httpx), dashboard (Rich), API (FastAPI) |
| Integracao | PyO3 + maturin | Fronteira Python/Rust com zero-copy |

## Estrutura

```text
CNPydge/
├── rust/cnpj_core/       # hot path: mmap, parser SIMD, normalizacao, pipeline, batch, destinos
├── python/
│   ├── app/               # inicializacao e composicao dos servicos
│   ├── features/          # funcionalidades por dominio (Vertical Slice)
│   │   ├── updater/       # download e atualizacao da base RFB
│   │   ├── company/       # consulta e manipulacao de CNPJ
│   │   ├── search/        # mecanismos de pesquisa
│   │   ├── export/        # exportacao (CSV, Parquet, DuckDB, Postgres)
│   │   ├── metrics/       # metricas e monitoramento
│   │   └── benchmark/     # benchmarks de desempenho
│   ├── shared/            # codigo compartilhado entre features
│   ├── integrations/      # adaptadores para servicos externos
│   │   ├── duckdb/
│   │   ├── postgres/
│   │   ├── receita/
│   │   └── storage/
│   ├── config/            # configuracao central (settings, logging, paths)
│   ├── entrypoints/       # interfaces de entrada (API, CLI, Dashboard)
│   └── tests/             # testes unitarios e integracao
├── docs/
│   ├── adr/               # Architecture Decision Records
│   ├── formato-rfb/       # notas sobre leiaute dos arquivos da RFB
│   └── benchmarks/        # resultados historicos de benchmarks
├── benchmarks/            # datasets e scripts de benchmark
├── docker/
├── scripts/
└── .mimocode/             # skills e agentes do MiMo Code
```

> **Nota**: diretorios legados (`python/api/`, `python/cli/`, `python/dashboard/`,
> `python/updater/`, `python/benchmark/`) existem para compatibilidade e serao migrados.
> Novas features devem seguir a estrutura acima. Ver `docs/ARCHITECTURE.md` para detalhes.

## Setup rapido

```bash
./scripts/bootstrap.sh
```

Requer Python 3.14 e Rust (rustup). O script cria o virtualenv, instala dependencias,
builda a extensao Rust via maturin e roda os testes.

## Roadmap

| Fase | Descricao | Status |
|---|---|---|
| 1 | Pipeline minimo (download -> parser -> COPY Postgres) | Em andamento |
| 2 | SIMD (memchr/csv-core/simdutf8) | Pendente |
| 3 | Apache Arrow (RecordBatch intermediario) | Pendente |
| 4 | DuckDB (destino de analytics) | Pendente |
| 5 | Atualizacao incremental (diffs mensais RFB) | Pendente |
| 6 | Dashboard (observabilidade visual) | Pendente |
| 7 | API REST (FastAPI) | Pendente |
| 8 | Benchmark automatizado (Criterion + CI) | Pendente |
| 9 | Plugin system (ClickHouse, Elastic, SQLite, Parquet, S3, BigQuery) | Pendente |

Ver `.mimocode/skills/core/planning/SKILL.md` para detalhes de cada fase.

## Arquitetura e decisoes

- `docs/ARCHITECTURE.md` -- visao arquitetural e estrutura alvo vs atual.
- `docs/adr/` -- Architecture Decision Records (por que cada decisao foi tomada).
- `docs/PHILOSOPHY.md` -- manifesto e principios de engenharia.
- `docs/VISION.md` -- visao do projeto e stack tecnologica.

## Agentes e skills

Este projeto usa o formato **MiMo Code** (`.mimocode/`):

- `skills/**/SKILL.md` -- doutrina de engenharia carregada sob demanda.
- `agents/*.md` -- subagentes especializados (arquiteto, rust builder, orquestrador python, etc.).

Ver `AGENTS.md` para a lista completa de agentes.

## Licensa

Proprietary -- ver `LICENSE` para detalhes.
