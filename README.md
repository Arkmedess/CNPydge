<div align="center">

# CNPydge

### Pipeline de alta vazão para os Dados Públicos de CNPJ da Receita Federal

Construído com **Rust** e **Python**

[![CI](https://img.shields.io/github/actions/workflow/status/Arkmedess/CNPydge/ci.yml?style=for-the-badge&label=CI&labelColor=1a1b26&color=7aa2f7)](https://github.com/Arkmedess/CNPydge/actions)
[![Python](https://img.shields.io/badge/Python-3.13+-3776AB?style=for-the-badge&logo=python&logoColor=white&labelColor=1a1b26)](https://www.python.org/)
[![Rust](https://img.shields.io/badge/Rust-2024_Edition-DEA584?style=for-the-badge&logo=rust&logoColor=white&labelColor=1a1b26)](https://www.rust-lang.org/)
[![Coverage](https://img.shields.io/badge/Coverage-94%25-9ece6a?style=for-the-badge&logo=pytest&logoColor=white&labelColor=1a1b26)](tests/)
[![License](https://img.shields.io/badge/License-MIT-9ece6a?style=for-the-badge&labelColor=1a1b26)](LICENSE)

</div>

---

## Por que o CNPydge?

A base pública de CNPJs da Receita Federal ultrapassa dezenas de gigabytes mensais e mais de 50 milhões de registros distribuídos em arquivos CSV compactados. A maioria dos pipelines existentes adota uma de duas abordagens frustrantes:

1. **Scripts em Python puro:** Sofrem com alto consumo de memória RAM e gargalo de GIL, levando horas para converter ou carregar os dados.
2. **Cargas pesadas em bancos relacionais:** Exigem subir instâncias de PostgreSQL ou MySQL apenas para fazer consultas analíticas que poderiam ser locais.

O **CNPydge** resolve isso dividindo as responsabilidades: um motor nativo em Rust que processa os arquivos em paralelo e gera Apache Parquet, mantendo uma interface amigável em Python para orquestrar downloads e pipelines analíticos.

## O que ele entrega

- **Conversão Direta para Parquet:** Gera arquivos colunares prontos para consulta instantânea em DuckDB, Polars, ClickHouse ou Pandas.
- **I/O com Mapeamento de Memória:** Lê arquivos massivos delegando paginação ao sistema operacional, sem estourar o limite de RAM da máquina.
- **Processamento Concorrente sem Bloqueio de GIL:** O motor em Rust opera em threads paralelas liberando totalmente o runtime do Python durante a ingestão pesada.
- **Esteira Unitária com Descarte Imediato (Rolling Eviction):** Processa e expurga arquivos temporários sob demanda, reduzindo a exigência de disco de ~70 GB para ~2 a 3 GB no pico.
- **Orquestração Descomplicada:** Utilitários em Python para download automatizado das tabelas da Receita (`Empresas`, `Estabelecimentos`, `Sócios`, etc.), retomada via checkpoints e verificação de integridade.

## Estrutura do Repositório

```text
CNPydge/
├── engine/              # Motor de alto desempenho em Rust (_core)
│   ├── src/             # Leitor mmap, parsers zero-copy e ArrowWriter
│   ├── benches/         # Harness de benchmarks com Criterion
│   └── README.md        # Documentação técnica do motor nativo
├── orchestrator/        # Pacote Python de orquestração (cnpydge)
│   ├── cnpydge/         # Crawler WebDAV, streaming downloader e wrapper FFI
│   │   └── pipeline/    # Esteira unitária, extração atômica, checkpoints e limpeza
│   └── README.md        # Documentação da API Python e utilitários
├── docs/                # Documentação técnica e governança
│   ├── adr/             # Registros de Decisão Arquitetural
│   └── architecture/    # Visão geral, fluxo de dados e catálogo de esquemas
├── CHANGELOG.md         # Histórico de versões
└── .github/workflows/   # CI determinístico com validação e testes
```

## Documentação

Toda a narrativa técnica, decisões e especificações formais do projeto estão organizadas em dois níveis estruturados:

- **Arquitetura & Fluxos:**
  - [Visão Geral da Arquitetura](docs/architecture/overview.md) — Separação de camadas e fronteiras FFI.
  - [Fluxo de Dados Ponta a Ponta](docs/architecture/data-flow.md) — Topologia de memória e sequência de ingestão.
  - [Catálogo de Esquemas Colunares](docs/architecture/schemas.md) — Tipagem Apache Arrow/Parquet das 10 tabelas da RFB.
- **Registros de Decisão Arquitetural (ADRs):**
  - [ADR-0001: Arquitetura Híbrida Rust e Python](docs/adr/0001-arquitetura-hibrida-rust-python.md)
  - [ADR-0002: Ingestão via Mmap e Zero-Copy com Liberação de GIL](docs/adr/0002-mmap-zero-copy-e-liberacao-de-gil.md)
  - [ADR-0003: Armazenamento Analítico Colunar em Parquet](docs/adr/0003-armazenamento-analitico-parquet.md)
  - [ADR-0004: Coleta Concorrente e Download Resumable via HTTP](docs/adr/0004-coleta-concorrente-e-download-resumable.md)
  - [ADR-0005: Suporte a CNPJ Alfanumérico e Coerção Estrita de Tipos](docs/adr/0005-cnpj-alfanumerico-e-coercao-de-tipos.md)
  - [ADR-0006: Ingestão por Esteira Unitária e Perfis de Armazenamento](docs/adr/0006-esteira-unitaria-rolling-eviction.md)
- **Módulos Locais:**
  - [Documentação do Motor Nativo em Rust](engine/README.md)
  - [Documentação do Orquestrador Python](orchestrator/README.md)
- **Histórico de Mudanças:**
  - [Changelog](CHANGELOG.md)

## Instalação e Uso Rápido

Para utilizar o CNPydge diretamente como pacote em seus projetos:

```bash
# Instalação direta no ambiente Python
uv pip install .

# Execução do script de demonstração integrado
uv run python main.py
```

### Exemplo de Uso Programático

```python
import cnpydge

# 1. Ingestão de ponta a ponta com esteira unitária (baixo uso de disco ~2-3 GB)
registros = cnpydge.run_pipeline(
    output_dir="./dados_parquet",
    tables=["empresas", "estabelecimentos"],
    profile=cnpydge.DiskRetentionProfile.BALANCED_ROLLING_EVICTION,
)

# 2. Conversão direta avulsa para Parquet (libera GIL)
total = cnpydge.to_parquet("Empresas0.csv", "empresas.parquet", kind="empresas")
print(f"Total de empresas convertidas: {total}")
```

## Desenvolvimento

Diretrizes para compilação local da extensão nativa e contribuição no projeto.

### Requisitos
- **Python:** `>= 3.13`
- **Rust:** `>= 1.85`
- **uv:** Gerenciador de pacotes e ambientes ([instalação](https://docs.astral.sh/uv/))

### Setup do Ambiente

```bash
git clone https://github.com/Arkmedess/CNPydge.git
cd CNPydge

# Inicializa o ambiente virtual e sincroniza as dependências
uv venv --python 3.13
uv sync

# Compila e instala a extensão nativa em modo de desenvolvimento
uv run --with maturin maturin develop
```

### Testes e Qualidade

- **Testes Python e Cobertura:** `uv run pytest --cov=cnpydge`
- **Testes do Motor Rust:** `cargo test` *(o `.cargo/config.toml` vincula o runtime ao Python 3.13 do `.venv`, prevenindo conflitos com o rust-analyzer)*
- **Linters e Análise Estática:** `uv run ruff check .` e `cargo clippy --all-targets`
- **Benchmarks de Throughput:** `cargo bench`

## Licença

Distribuído sob a licença MIT. Veja `LICENSE` para mais detalhes.