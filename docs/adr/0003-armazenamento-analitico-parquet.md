# 3. Armazenamento Analítico Colunar em Apache Parquet

Data: 2026-09-18
Status: Aceito

## Contexto
A base pública do CNPJ é utilizada majoritariamente para cargas analíticas (OLAP), consultas agregadas, cruzamentos de inteligência fiscal e enriquecimento cadastral. As abordagens tradicionais costumam exigir:
1. Provisionamento de instâncias pesadas de SGBDs transacionais (PostgreSQL/MySQL), gerando alto custo operacional de infraestrutura e índices lentos para carregar >50 milhões de linhas.
2. Formatos brutos (CSV) que exigem varredura completa e custoso re-parsing a cada nova consulta.

## Decisão
Adotar o padrão aberto **Apache Parquet** com esquemas formais em **Apache Arrow** e compressão **Snappy** como formato de armazenamento persistente do pipeline:
1. **Esquema Tipado Estrito:** Cada tipo de tabela do CNPJ possui um `SchemaRef` explícito do Arrow com tipos primitivos compactos (`UInt8`, `UInt16`, `UInt32`, `Float64`, `Utf8`), reduzindo drasticamente o footprint de dados.
2. **Processamento Concorrente com Rayon:** Os blocos de bytes fatiados são transformados em `RecordBatch` em paralelo através do pool de threads do `Rayon`.
3. **Escrita Única Otimizada:** Os `RecordBatch` gerados concorrentemente são consolidados e serializados em um único arquivo Parquet via `ArrowWriter` com compressão Snappy.

## Consequências
- **Ganhos:**
  - Redução de tamanho de 60% a 80% em relação aos CSVs descompactados originais.
  - Compatibilidade imediata e nativa com ferramentas analíticas modernas (*DuckDB*, *Polars*, *ClickHouse*, *Pandas* e *Apache Spark*) sem necessitar de banco de dados em execução.
  - Leitura colunar com projeção de campos (*projection pushdown*) e filtros no metadado (*predicate pushdown*).
- **Compromissos:**
  - Formato imutável voltado para OLAP; não suporta mutações pontuais em tempo real (*updates/deletes* linha a linha) comuns em bancos relacionais (OLTP).
  - Necessidade de conversão prévia em lote antes da disponibilização para consulta.

