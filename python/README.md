# Camada Python - Orquestração

Camada de orquestração do CNPydge em Python 3.14. Responsável por CLI, configuração, download, dashboard e API.

## Arquitetura

```
python/
├── app/                 # Inicialização e composição dos serviços
├── features/            # Funcionalidades por domínio (Vertical Slice)
│   ├── updater/         # Download e atualização da base RFB
│   ├── company/         # Consulta e manipulação de CNPJ
│   ├── search/          # Mecanismos de pesquisa
│   ├── export/          # Exportação (CSV, Parquet, DuckDB, Postgres)
│   ├── metrics/         # Métricas e monitoramento
│   └── benchmark/       # Benchmarks de desempenho
├── shared/              # Código compartilhado entre features
├── integrations/        # Adaptadores para serviços externos
│   ├── duckdb/
│   ├── postgres/
│   ├── receita/
│   └── storage/
├── config/              # Configuração central (settings, logging, paths)
├── entrypoints/         # Interfaces de entrada (API, CLI, Dashboard)
└── tests/               # Testes unitários e integração
```

## Princípios

1. **Vertical Slice**: Cada feature é um slice vertical, com limites claros entre domínio e infraestrutura
2. **Separação de responsabilidades**: Python cuida da orquestração, Rust do hot path
3. **Configuração centralizada**: Pydantic Settings com suporte a TOML e variáveis de ambiente
4. **Async por padrão**: Download e API usam async/await

## Setup

```bash
# Instalar dependências
uv sync

# Build da extensão Rust
uv run maturin develop --release

# Rodar testes
uv run pytest
```

## Uso

### CLI (Typer)

```bash
# Importar base completa
uv run python -m python.entrypoints.cli importar

# Importar arquivos específicos
uv run python -m python.entrypoints.cli importar --files Empresas0.zip

# Ver status do cache
uv run python -m python.entrypoints.cli status
```

### Configuração

O sistema usa Pydantic Settings com três fontes (em ordem de prioridade):

1. Variáveis de ambiente com prefixo `CNPJ_`
2. Arquivo `config.toml`
3. Arquivo `.env`

Exemplo de `config.toml`:

```toml
postgres_dsn = "postgresql://user:pass@localhost:5432/cnpj"
batch_size = 100_000
download_dir = "./data/cache"
log_level = "INFO"
```

Exemplo de variável de ambiente:

```bash
export CNPJ_POSTGRES_DSN="postgresql://user:pass@localhost:5432/cnpj"
export CNPJ_LOG_LEVEL="DEBUG"
```

### API REST (FastAPI)

```bash
# Iniciar servidor
uv run uvicorn python.entrypoints.api:app --reload

# Endpoints disponíveis
GET /health          # Health check
GET /status          # Status da importação
POST /import         # Iniciar importação
```

## Features

### updater (Download)

Download assíncrono dos arquivos da RFB com:

- Cache local (não re-download de arquivos já baixados)
- Verificação SHA256
- Proteção contra zip bomb (limite de 10GB descompactado)
- Download concorrente (4 conexões simultâneas)

```python
from python.features.updater.downloader import download_all

downloaded = await download_all(
    dest_dir=Path("./data/cache"),
    files=["Empresas0.zip"],
    max_concurrent=4,
)
```

### company (Consulta)

Consulta e manipulação de dados de CNPJ (preparado para Fase 7).

### search (Pesquisa)

Mecanismos de pesquisa sobre a base de dados (preparado para Fase 7).

### export (Exportação)

Exportação da base para diferentes formatos (preparado para Fases 3-4).

### metrics (Métricas)

Coleta e exposição de métricas de performance do pipeline.

### benchmark (Benchmarks)

Benchmarks de desempenho do pipeline (preparado para Fase 8).

## Testes

```bash
# Todos os testes
uv run pytest

# Testes específicos
uv run pytest python/tests/test_downloader.py

# Com verbose
uv run pytest -v --tb=long
```

## Estrutura de testes

```
python/tests/
├── __init__.py
├── test_downloader.py    # Testes do módulo de download
├── test_settings.py      # Testes de configuração
└── test_cli.py           # Testes da CLI
```

## Dependências principais

| Pacote | Versão | Uso |
|---|---|---|
| `typer` | ≥0.15 | CLI |
| `pydantic` | ≥2.9 | Validação de dados |
| `pydantic-settings` | ≥2.6 | Configuração |
| `rich` | ≥13.9 | Dashboard e formatação |
| `loguru` | ≥0.7 | Logging |
| `httpx` | ≥0.28 | Download assíncrono |
| `fastapi` | ≥0.115 | API REST |
| `uvicorn` | ≥0.32 | Servidor ASGI |

## Licença

Proprietary -- ver `LICENSE` para detalhes.
