# 1. Adoção de Arquitetura Híbrida Rust (PyO3) e Python (uv)

Data: 2026-09-17
Status: Aceito

## Contexto
O processamento da base pública de CNPJs da Receita Federal (>50 milhões de registros, dezenas de gigabytes mensais em CSV compactado) impõe duas demandas conflitantes:
1. Necessidade de vazão extrema de I/O, baixo consumo de memória e execução paralela sem gargalo de GIL.
2. Necessidade de ergonomia, facilidade de orquestração de rede (WebDAV/HTTP), scripts de automação e integração fluida com o ecossistema analítico.

Pipelines em Python puro sofrem com overhead de alocação de objetos e bloqueio de threads pela GIL. Por outro lado, ferramentas puramente em linguagens de sistemas reduzem a adesão da comunidade analítica e complicam scripts de automação.

## Decisão
Adotar uma arquitetura híbrida dividida em duas camadas complementares:
- **Motor Nativo (`engine/`):** Desenvolvido em Rust com `PyO3` e compilado via `maturin`, responsável pela leitura de arquivos, parsing de registros e escrita colunar em Apache Parquet.
- **Camada de Orquestração (`orchestrator/`):** Desenvolvida em Python (>= 3.13) gerenciada via `uv`, responsável pela descoberta de arquivos via WebDAV, downloads concorrentes resilientes com retomada (`Range`), logging estruturado e interface pública unificada.

## Consequências
- **Ganhos:**
  - Liberação completa do GIL do Python (`py.allow_threads`) durante o processamento intensivo de dados.
  - Eficiência de memória com tipagem estática e ausência de garbage collector na camada pesada.
  - Distribuição ergonômica como pacote Python padrão (`import cnpydge`).
  - Gerenciamento determinístico de dependências e ambientes com `uv`.
- **Compromissos:**
  - Exigência da toolchain Rust (>= 1.85) e `maturin` para compilação local e pipelines de CI/CD.
  - Necessidade de sincronizar as declarações de tipagem (`_core.pyi`) com as funções exportadas em Rust.

