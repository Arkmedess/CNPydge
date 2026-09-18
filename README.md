<div align="center">

# CNPydge

### Pipeline de alta vazão para os Dados Públicos de CNPJ da Receita Federal

Construído com **Rust** e **Python**

[![CI](https://img.shields.io/github/actions/workflow/status/Arkmedess/CNPydge/ci.yml?style=for-the-badge&label=CI&labelColor=1a1b26&color=7aa2f7)](https://github.com/Arkmedess/CNPydge/actions)
[![Python](https://img.shields.io/badge/Python-3.13+-3776AB?style=for-the-badge&logo=python&logoColor=white&labelColor=1a1b26)](https://www.python.org/)
[![Rust](https://img.shields.io/badge/Rust-2024_Edition-DEA584?style=for-the-badge&logo=rust&logoColor=white&labelColor=1a1b26)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-9ece6a?style=for-the-badge&labelColor=1a1b26)](LICENSE)

</div>

---

## Por que o CNPydge?

A base pública de CNPJs da Receita Federal ultrapassa dezenas de gigabytes mensais e mais de 50 milhões de registros distribuídos em arquivos CSV compactados. A maioria dos pipelines existentes adota uma de duas abordagens frustrantes:

1. **Scripts em Python puro:** Sofrem com alto consumo de memória RAM e gargalo de GIL, levando horas para converter ou carregar os dados.
2. **Cargas pesadas em bancos relacionais:** Exigem subir instâncias de PostgreSQL ou MySQL apenas para fazer consultas analíticas que poderiam ser locais.

O **CNPydge** resolve isso dividindo as responsabilidades: um motor nativo em Rust que processa os arquivos em paralelo e gera **Apache Parquet**, mantendo uma interface amigável em Python para orquestrar downloads e pipelines analíticos.

## O que ele entrega

- **Conversão Direta para Parquet:** Gera arquivos colunares prontos para consulta instantânea em DuckDB, Polars, ClickHouse ou Pandas.
- **I/O com Mapeamento de Memória:** Lê arquivos massivos delegando paginação ao sistema operacional, sem estourar o limite de RAM da máquina.
- **Processamento Concorrente sem Bloqueio de GIL:** O motor em Rust opera em threads paralelas liberando totalmente o runtime do Python durante a ingestão pesada.
- **Orquestração Descomplicada:** Utilitários em Python para download automatizado das tabelas da Receita (`Empresas`, `Estabelecimentos`, `Sócios`, etc.) e verificação de integridade.

## Estrutura do Repositório

```text
CNPydge/
├── engine/              # Motor de alto desempenho em Rust (_core)
│   ├── src/lib.rs       # Interface nativa e bindings PyO3
│   └── benches/         # Harness de benchmarks com Criterion
├── orchestrator/        # Pacote Python de orquestração (cnpydge)
│   └── cnpydge/         # Pipelines de download, CLI e utilitários
├── docs/                # Decisões de arquitetura (ADRs) e especificações
└── .github/workflows/   # CI determinístico com validação e testes
```

## Começando

### Requisitos
- **Python:** `>= 3.13`
- **Rust:** `>= 1.85`
- **uv:** Gerenciador de ambiente e pacotes ([instalação](https://docs.astral.sh/uv/))

### Instalação Local

1. Clone o repositório:
```bash
git clone https://github.com/Arkmedess/CNPydge.git
cd CNPydge
```

2. Inicialize o ambiente virtual e compile a extensão nativa:
```bash
uv venv --python 3.13
uv run --with maturin maturin develop
```

3. Valide a execução:
```bash
uv run python main.py
```

## Desenvolvimento e Testes

- **Executar testes:** `cargo test`
- **Linters e Análise Estática:** `cargo fmt --all --check && cargo clippy --all-targets`
- **Benchmarks de Throughput:** `cargo bench`

## Licença

Distribuído sob a licença MIT. Veja `LICENSE` para mais detalhes.

