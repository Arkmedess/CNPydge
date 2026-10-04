# Changelog

Todas as alterações notáveis neste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/),
e este projeto adere ao [Semantic Versioning](https://semver.org/lang/pt-BR/).

## [Unreleased]

### Adicionado
- **Sanitização Ativa de Logs e Observabilidade (`cnpydge.logging`):**
  - Filtro `SensitiveDataFilter` para mascaramento automático de credenciais em URLs, tokens Bearer, senhas e CPFs.
  - Formatador `IsoUtcFormatter` com timestamps em UTC estrito no padrão RFC 3339 / ISO 8601.
  - Formatador `JsonLogFormatter` para exportação de eventos em JSON estruturado para observabilidade.
  - Inicialização idempotente e thread-safe de logging via `setup_logging()`.
- **Prevenção Rigorosa de Bugs na Suíte de Testes:**
  - Verificação de integridade colunar validando *magic bytes* `PAR1` nos arquivos Parquet gerados.
  - Testes de segurança contra ataques de extração (*Zip Slip*) e arquivos compactados truncados/corrompidos.
  - Cenários de falhas de rede WebDAV, tamanhos não-numéricos no XML e fallback para raiz.

### Modificado
- **Refatoração Pragmática & Redução de Complexidade:**
  - `WebDavCrawler`: Unificação de consulta WebDAV entre `list_files_by_period` e `catalog`, eliminando duplicação de requisições `PROPFIND` e adotando filtro em passada única.
  - `downloader`: Decomposição de `download_file` com funções auxiliares puras e *guard clauses*, reduzindo complexidade ciclomática de 12 para 3.
  - `disk_cleanup_policy`: Centralização de descarte atômico via `path.unlink(missing_ok=True)` eliminando checagens TOCTOU manuais.
- **Otimizações no Motor Nativo Rust (`cnpydge-core`):**
  - `parse_capital` em `empresa.rs`: Conversão monetária *in-place* com buffer de pilha (`[0u8; 32]`), garantindo **zero-allocation** no heap para as ~60 milhões de linhas de empresas da RFB.
  - `split_two_fields` em `dominio.rs`: Adoção dos métodos idiomáticos `strip_suffix` e `iter().position(...)`.
  - `converter.rs`: Unificação de tabelas de domínio (`u32` e `u16`) via macro declarativa `build_dominio_batch!`.
  - `split_csv_line` em `util.rs`: Divisão atômica e *zero-copy* de linhas CSV via *const generics*, eliminando laços manuais e duplicação em `empresa.rs`, `socio.rs`, `estabelecimento.rs` e `simples.rs`.
  - `main.py`: Parametrização declarativa dos casos de demonstração via `DEMO_CASES`, eliminando blocos repetidos de teste.

## [0.2.0] - 2026-09-30

### Adicionado
- **Esteira Unitária com Descarte Imediato (`Rolling Eviction`):** Módulo `cnpydge.pipeline` e orquestrador `RfbPipelineOrchestrator` implementando o ciclo de vida completo de descoberta remota, download, extração atômica, conversão nativa e expurgo pontual.
- **Redução Drástica de Armazenamento:** Redução da exigência de disco temporário de ~70 GB para ~2 a 3 GB no pico operacional (apenas 1 partição por vez).
- **Perfis de Retenção Configuráveis (`DiskRetentionProfile`):**
  - `BALANCED_ROLLING_EVICTION` (Padrão): Expurga o `.zip` logo após a extração e o `.csv` após a escrita do Parquet.
  - `PERFORMANCE_BATCH_RETAIN`: Preserva intermediários para execução de alto rendimento em instâncias com storage abundante.
  - `ARCHIVAL_PERMANENT_ZIP`: Preserva os arquivos `.zip` originais da Receita Federal para histórico e auditoria.
- **Manifesto de Checkpoints Idempotente (`JsonCheckpointManifestStore`):** Rastreabilidade atômica dos estágios operacionais (`PENDING_DOWNLOAD`, `DOWNLOAD_COMPLETED`, `EXTRACTION_COMPLETED`, `CONVERSION_COMPLETED`, `PROCESSING_FAILED`) viabilizando retentativas sem reprocessamento.
- **Extração Atômica e Segura (`StandardZipArchiveExtractor`):** Descompressão em streaming direto para arquivo `.csv` intermediário, prevenindo corrupção e artefatos parciais.
- **Descoberta WebDAV Automatizada (`WebDavCrawler.catalog`):** Método de consulta de alto nível com identificação cronológica automática do período mais recente da RFB e suporte a padrões glob.
- **Função Pública de Ingestão (`cnpydge.run_pipeline`):** Ponto de entrada amigável na API Python para execução da esteira completa com filtros de tabela e limites de partição.
- **Decisão Arquitetural Formalizada:** Registro formal do [ADR-0006: Ingestão por Esteira Unitária e Perfis de Armazenamento](docs/adr/0006-esteira-unitaria-rolling-eviction.md).

## [0.1.0] - 2026-09-20

### Adicionado
- **Motor Nativo em Rust (`engine`):** Extensão compilada via PyO3/maturin para parsing e conversão colunar em Apache Parquet com liberação explícita de GIL (`py.allow_threads`).
- **I/O de Alta Performance:** Leitor mapeado em memória virtual (`memmap2`) com particionamento zero-copy de fatias contíguas alinhadas a quebras de linha (`\n`).
- **Suporte às 10 Tabelas da RFB:** Conversor dedicado para `Empresas`, `Sócios`, `Estabelecimentos`, `Simples Nacional/MEI`, `CNAEs`, `Motivos`, `Municípios`, `Naturezas`, `Países` e `Qualificações`.
- **Suporte a CNPJ Alfanumérico:** Validação estrita preservando compatibilidade com o novo formato de identificação da Receita Federal.
- **Tipagem Analítica Compacta:** Coerção de datas para `UInt32` (`YYYYMMDD`), valores monetários para `Float64` e códigos de tabelas auxiliares para `UInt8`/`UInt16`/`UInt32` com compressão Snappy.
- **Descoberta WebDAV (`WebDavCrawler`):** Consulta recursiva via HTTP `PROPFIND` com tolerância a múltiplos namespaces XML e identificação cronológica de períodos.
- **Download Concorrente e Resumable (`downloader`):** Streaming HTTP em blocos de 1 MB, escrita atômica em arquivos parciais `.part` e suporte a retomada via cabeçalho `Range`.
- **Base Documental de Arquitetura:**
  - 5 Registros de Decisão Arquitetural (ADRs de 0001 a 0005 no formato Nygard).
  - Especificação de arquitetura, fluxo de dados e diagramas visuais em `docs/architecture/`.
  - Catálogo formal de esquemas colunares Arrow em `docs/architecture/schemas.md`.
  - Documentação local de módulos em `engine/README.md` e `orchestrator/README.md`.

