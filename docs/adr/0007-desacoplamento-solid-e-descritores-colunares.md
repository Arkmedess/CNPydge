# 7. Desacoplamento Arquitetural SOLID e Descritores Colunares Modulares

Data: 2026-10-13  
Status: Aceito  

## Contexto

À medida que o projeto CNPydge amadureceu com a esteira unitária de ingestão e observabilidade endurecida, a análise arquitetural sob os princípios SOLID identificou dois pontos críticos de acoplamento:

1. **Centralização Excessiva no Motor de Conversão (`converter.rs`):**
   O módulo acumulava regras de parsing de fatias, declaração manual de esquemas Apache Arrow e montagem de `RecordBatch` para todas as tabelas suportadas da Receita Federal (>760 linhas). Qualquer alteração de layout ou nova entidade exigia editar esse arquivo central, violando o Princípio de Responsabilidade Única (SRP) e o Princípio Aberto/Fechado (OCP).

2. **Acoplamento Direto no Orquestrador (`rfb_pipeline_orchestrator.py`):**
   O orquestrador instanciava e invocava diretamente `WebDavCrawler` para descobrir partições remotas. Isso impedia a injeção de fontes alternativas (armazenamento local, S3, mock para testes rápidos) e violava o Princípio da Inversão de Dependência (DIP) e o Princípio de Substituição de Liskov (LSP). Além disso, os mapeamentos de prefixos de tabelas estavam dispersos no crawler, com falhas na identificação de arquivos plurais (`Qualificacoes.zip`).

## Decisão

Implementar o desacoplamento formal de ambas as fronteiras arquiteturais:

### 1. Fronteira de Descoberta e Catálogo
- **`RfbTableRegistry`:** Centralizar as regras de mapeamento de layout, padrões de arquivos (`*.zip`) e prefixos canônicos da Receita Federal em uma entidade declarativa coesa em `pipeline/partition_models.py`.
- **Protocolo `PartitionDiscoveryProvider`:** Declarar um protocolo abstrato estrutural em `pipeline/pipeline_contracts.py` definindo `discover_partitions(base_url, period, tables) -> list[RfbPartition]`.
- **`WebDavPartitionDiscoveryProvider`:** Implementar o provedor concreto WebDAV isolado em `pipeline/webdav_partition_discovery.py`.
- **Inversão de Dependência no Orquestrador:** `RfbPipelineOrchestrator` passa a receber `discovery_provider: PartitionDiscoveryProvider | None = None` no construtor com fallback automático, desacoplando-se de bibliotecas de transporte HTTP.

### 2. Fronteira de Conversão Colunar
- **Trait `TableDescriptor`:** Declarar o contrato polimórfico em `converter.rs`:
  ```rust
  pub trait TableDescriptor: Send + Sync {
      fn name(&self) -> &'static str;
      fn schema(&self) -> SchemaRef;
      fn build_batch(&self, slice: &[u8]) -> Result<Option<RecordBatch>, ConvertErr>;
  }
  ```
- **Descritores Descentralizados:** Cada módulo de domínio passa a conter seu próprio descritor estático e encapsular seu esquema Arrow e montagem de lote:
  - `EmpresaDescriptor` em `empresa.rs`
  - `SocioDescriptor` em `socio.rs`
  - `EstabelecimentoDescriptor` em `estabelecimento.rs`
  - `SimplesDescriptor` em `simples.rs`
  - `DominioDescriptor<K>` parametrizado em `dominio.rs` para tabelas de apoio (`u16` e `u32`).
- **Resolução Polimórfica:** `TableKind::descriptor(&self) -> &'static dyn TableDescriptor` delega chamadas de `schema()` e `build_batch()`, reduzindo `converter.rs` para apenas 321 linhas estritamente dedicadas ao pipeline streaming (`MmapReader` + `Rayon` + `ArrowWriter`).

## Consequências

- **Ganhos:**
  - **Extensibilidade (OCP):** A inclusão de uma nova tabela exige apenas criar seu módulo com a trait `TableDescriptor`, sem intervenção no pipeline Rayon/Mmap.
  - **Flexibilidade de Ingestão (DIP/LSP):** O orquestrador agora pode ser acoplado a qualquer provedor de descoberta (S3, disco local, WebDAV ou mocks em memória) sem alterar uma linha de orquestração.
  - **Manutenibilidade:** Eliminação de centenas de linhas de código procedural monolítico e macros colunares dispersas.
  - **Zero Regressão:** Retrocompatibilidade estrita das funções públicas (`to_parquet`, `run_pipeline`) e integridade de tipos preservada.
- **Compromissos:**
  - Introdução de pequenas estruturas de descritores (*zero-sized types* `EmpresaDescriptor`, `SocioDescriptor`, etc.) que servem como pontos de ancoragem estáticos para a trait `TableDescriptor`.
