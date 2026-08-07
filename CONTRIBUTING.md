# Guia de Contribuição

Obrigado por contribuir com o CNPydge! Este documento explica como participar do desenvolvimento.

## Visão Geral

O CNPydge é um pipeline de ingestão de alta performance para dados de CNPJ da Receita Federal.
O projeto usa **Python 3.14** para orquestração e **Rust** para o hot path (via PyO3/maturin).

## Pré-requisitos

- **Python 3.14+**: https://www.python.org/downloads/
- **Rust**: https://rustup.rs/
- **uv**: https://docs.astral.sh/uv/ (gerenciador de pacotes Python)
- **PostgreSQL**: Para testes de integração

## Setup do Ambiente

```bash
# 1. Clonar o repositório
git clone https://github.com/Arkmedess/CNPydge.git
cd CNPydge

# 2. Executar bootstrap (instala dependências + build Rust)
./scripts/bootstrap.sh

# 3. Verificar se tudo funciona
uv run pytest
cargo test
```

## Estrutura do Projeto

```
CNPydge/
├── rust/cnpj_core/       # Hot path em Rust (parser, normalização, pipeline)
├── python/               # Camada de orquestração Python
│   ├── features/         # Funcionalidades por domínio (Vertical Slice)
│   ├── entrypoints/      # CLI, API, Dashboard
│   ├── config/           # Configuração central
│   └── tests/            # Testes Python
├── docs/                 # Documentação e ADRs
├── scripts/              # Scripts de automação
└── benchmarks/           # Datasets e scripts de benchmark
```

## Regras de Contribuição

### 1. Commits

Use **Conventional Commits** em inglês para rastreabilidade:

```
feat: add parquet export support
fix: fix parsing of alphanumeric CNPJ
docs: update Rust module README
refactor: extract download logic to separate module
test: add normalization tests
perf: optimize delimiter search with SIMD
```

### 2. Branches

- `main`: Produção, sempre estável
- `feat/nome-da-feature`: Features novas
- `fix/nome-do-fix`: Correções de bugs
- `docs/nome-da-doc`: Atualizações de documentação

### 3. Pull Requests

1. Crie uma branch a partir de `main`
2. Implemente a sua mudança
3. Adicione testes se aplicável
4. Atualize a documentação se necessário
5. Abra um PR com título claro e descrição detalhada

### 4. Código

#### Python

- **Formatter**: `ruff format` (line-length=100)
- **Linter**: `ruff check`
- **Type checker**: `mypy --strict`
- **Testes**: `pytest -v --tb=long --strict-markers -x`

```bash
# Formatar
uv run ruff format python/

# Verificar
uv run ruff check python/

# Type check
uv run mypy python/

# Testes
uv run pytest
```

#### Rust

- **Formatter**: `rustfmt`
- **Linter**: `clippy`
- **Testes**: `cargo test`

```bash
# Formatar
cargo fmt

# Verificar
cargo clippy -- -D warnings

# Testes
cargo test

# Benchmarks
cargo bench
```

### 5. Documentação

- **Português**: A documentação deve ser em português e os commits no padrão inglês (projeto focado em dados brasileiros)
- **ADRs**: Decisões de arquitetura devem ser documentadas em `docs/adr/`
- **READMEs**: Cada módulo deve ter um README com responsabilidade, uso e testes

### 6. Princípios de Engenharia

O projeto segue 10 princípios inegociáveis:

1. **Zero-copy**: Dados lidos diretamente do mmap, sem cópias
2. **Streaming**: Processamento registro a registro
3. **Memória constante**: Uso de RAM independente do tamanho do arquivo
4. **Hot path em Rust**: Parsing e normalização sempre em Rust
5. **Fronteira Python/Rust**: PyO3 com GIL release
6. **Sem panic no hot path**: Todos os erros retornam `Result`
7. **Backpressure**: Canais bounded com crossbeam
8. **Observabilidade**: Métricas e tracing em todas as etapas
9. **Modularidade**: Novos destinos entram como traits (Fase 9)
10. **Testabilidade**: Cobertura de testes unitários e integração

Ver `docs/PHILOSOPHY.md` e `.mimocode/skills/core/engineering-principles/SKILL.md` para detalhes.

## Ciclo de Desenvolvimento

### 1. Planejamento

Antes de implementar uma feature:

1. Verifique o roadmap em `.mimocode/skills/core/planning/SKILL.md`
2. Crie um ADR se a decisão for significativa
3. Use o template de plano por feature

### 2. Implementação

1. Crie uma branch `feat/nome-da-feature`
2. Implemente com testes
3. Documente decisões importantes
4. Execute lint e testes antes de commitar

### 3. Revisão

1. Abra um PR
2. Peça revisão de pelo menos 1 pessoa
3. Resolva feedback
4. Merge após aprovação

### 4. Deploy

1. Merge em `main`
2. Tags de versão seguem semântico (`v0.1.0`, `v0.2.0`, etc.)

## Issues e Bugs

Ao reportar um bug, inclua:

1. **Descrição**: O que aconteceu vs. o que era esperado
2. **Passos para reproduzir**: Comandos e dados usados
3. **Ambiente**: SO, versão do Python, versão do Rust
4. **Logs**: Saída completa do erro
5. **Screenshots**: Se aplicável

## Discussões Arquiteturais

Decisões significativas devem ser discutidas em Issues antes de implementadas:

- Mudanças na fronteira Python/Rust
- Novos destinos de dados
- Mudanças no formato dos dados
- Alterações no pipeline de processamento

Use o template de ADR em `docs/adr/` para documentar decisões.

## Contato

- **Issues**: Para bugs e features
- **Discussions**: Para dúvidas e discussões gerais
- **PRs**: Para contribuições diretas

## Licença

Ao contribuir, você concorda que suas contribuições serão licenciadas sob a mesma licença do projeto.
