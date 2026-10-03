# Changelog

Todas as alterações notáveis neste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/),
e este projeto adere ao [Semantic Versioning](https://semver.org/lang/pt-BR/).

## [Unreleased]

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

