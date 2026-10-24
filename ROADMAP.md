# Roadmap de Engenharia do CNPydge

Documento orientador de capacidades técnicas, evolução arquitetural e governança de releases SemVer do **CNPydge**.

---

## 🎯 Visão Executiva & Régua SemVer

```text
v0.1.0 (Fundação) ──► v0.2.0 (Esteira Unitária) ──► v0.3.0 (CLI & Observabilidade) ──► v0.4.0 (Sinks Analíticos) ──► v1.0.0 (GA / Produção)
```

| Milestone | Foco Principal | Status | Marco |
| :--- | :--- | :--- | :--- |
| **[v0.1.0 - Setup & Fundação](#v010---setup--fundação)** | Motor Rust (`_core`), mmap zero-copy, suporte às 10 tabelas da RFB e crawler WebDAV. | ✅ Concluído | 2026-09-20 |
| **[v0.2.0 - Pipeline Integrado & Esteira Unitária](#v020---pipeline-integrado--esteira-unitária)** | Ingestão unitária sequencial, rolling eviction (~2-3 GB disco no pico) e checkpoints JSON. | ✅ Concluído | 2026-10-03 |
| **[v0.3.0 - Interface de Linha de Comando (CLI) & Observabilidade](#v030---interface-de-linha-de-comando-cli--observabilidade)** | Reorganização de pacotes (`network`, `api`), manifesto formal `catalog.json` e CLI Typer. | 🚧 Em Andamento | 2026-Q4 |
| **[v0.4.0 - Carga Real RFB, Resiliência & Profiling](#v040---carga-real-rfb-resiliência--profiling)** | Sinks DuckDB, PostgreSQL (streaming COPY), Object Storage (S3/R2) e profiling com base real. | 📋 Planejado | 2026-Q4 |
| **[v1.0.0 - Lançamento Oficial (Production-Ready)](#v100---lançamento-oficial-production-ready)** | Disponibilidade geral (GA), wheels binárias multiplataforma no PyPI e contratos estáveis. | 🔒 Bloqueado | 2027-Q1 |

---

## Detalhamento das Milestones

### v0.1.0 - Setup & Fundação
**Objetivo:** Estabelecer a arquitetura híbrida Rust/Python com motor nativo leitor de CSV via memória mapeada (`mmap`) e conversão para Apache Parquet colunar.

- [x] **#1** - `feat(engine): implementar leitor de arquivos CSV via mmap zero-copy em Rust`
- [x] **#2** - `feat(engine): suporte a tipos de dados específicos do CNPJ e alfanumérico (ADR-0005)`
- [x] **#3** - `feat(crawler): descoberta e catalogação de arquivos no repositório WebDAV da RFB`
- [x] **#4** - `feat(downloader): download concorrente em streaming com suporte a retomada HTTP Range`
- [x] **#5** - `ci(actions): configurar matriz de CI para Rust e Python`

---

### v0.2.0 - Pipeline Integrado & Esteira Unitária
**Objetivo:** Eliminar o gargalo de armazenamento em disco reduzindo a exigência de ~70 GB para ~2 a 3 GB no pico através de esteira unitária com descarte imediato (*Rolling Eviction*).

- [x] **#6** - `feat(pipeline): implementar esteira unitária com ciclo download -> extração -> conversão -> expurgo`
- [x] **#7** - `feat(pipeline): sistema de persistência de checkpoints idempotente em JSON`
- [x] **#22** - `feat(logging): telemetria estruturada, formatação RFC 3339 e sanitização ativa de segredos`
- [x] **#23** - `refactor(solid): desacoplamento arquitetural e descritores colunares modulares (ADR-0007)`

---

### v0.3.0 - Interface de Linha de Comando (CLI) & Observabilidade
**Objetivo:** Fornecer interface de terminal rica para operadores, manifesto formal de dados como contrato de entrega e modularização de fronteiras.

- [x] **#8** - `refactor(orchestrator): reorganizar modulos em api, network e catalog mantendo fachada publica`
- [ ] **#9** - `feat(catalog): implementar manifesto formal de dados (catalog.json) como contrato de entrega`
- [ ] **#10** - `feat(cli): criar interface de terminal com Typer e subcomandos (run, convert, download, catalog, inspect)`
- [ ] **#11** - `test(cli): adicionar suite de testes de interface de terminal e validacao de exit codes`

**Critérios de Saída da Milestone (DoD):**
1. 100% das 4 issues fechadas via Pull Requests aprovados.
2. Suíte de testes com cobertura $\ge 90\%$ e zero avisos de linter (`ruff`, `clippy`).
3. CLI validada com subcomandos modulares e suporte a `--json`, `--help` e `NO_COLOR`.

---

### v0.4.0 - Carga Real RFB, Resiliência & Profiling
**Objetivo:** Expandir a capacidade de exportação analítica para múltiplos destinos e comprovar robustez sob ensaios de estresse com a base completa da Receita Federal.

- [ ] **#12** - `feat(sinks): implementar contrato abstrato DataSink e conector para DuckDB`
- [ ] **#13** - `feat(sinks): implementar conector de alta vazao para PostgreSQL via streaming COPY binario`
- [ ] **#14** - `feat(sinks): implementar conector de nuvem para Object Storage (S3 / Cloudflare R2) com particionamento Hive`
- [ ] **#15** - `perf(profiling): executar ensaios de estresse com base real da RFB e validar pico de disco e memoria`

---

### v1.0.0 - Lançamento Oficial (Production-Ready)
**Objetivo:** Disponibilidade geral (GA) com congelamento SemVer, publicação no PyPI com compilação cruzada e distribuição de contêiner oficial.

- [ ] **#16** - `ci(release): configurar matriz cibuildwheel e publicacao automatizada de wheels no PyPI`
- [ ] **#17** - `docs(guides): elaborar especificacoes de integracao para sites analiticos e tutorial de consumo da API`
- [ ] **#18** - `chore(semver): auditoria de contratos de API publica e congelamento SemVer v1.0.0`
- [ ] **#24** - `ci(docker): automatizar build multi-arch e publicacao de imagem oficial no GHCR a cada release`

---

## Governança e Ciclo de Vida

Para detalhes sobre regras SemVer, critérios de fechamento de milestones e procedimentos de publicação de tags anotadas, consulte a [Política de Versionamento e Governança de Releases](docs/governance/release-and-versioning-policy.md).

