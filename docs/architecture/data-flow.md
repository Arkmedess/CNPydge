# Fluxo de Dados Ponta a Ponta

Este documento descreve o ciclo de vida dos dados no **CNPydge**, desde a consulta ao repositório público da Receita Federal até a persistência em formato analítico colunar.

## 1. Topologia de Dados e Gestão de Memória

O diagrama a seguir detalha a jornada dos bytes através dos subsistemas do CNPydge, destacando as fronteiras de memória virtual, isolamento de heap e processamento paralelo:

```mermaid
flowchart TD
    subgraph S1["1. Descoberta Remota (WebDAV)"]
        RFB["Servidor WebDAV da Receita Federal"] -->|"PROPFIND (HTTP)"| Crawl["WebDavCrawler"]
        Crawl -->|"Cataloga metadados"| Meta["Lista de RemoteFileMetadata\n(Nome, Tamanho, Data)"]
    end

    subgraph S2["2. Ingestão de Rede Concorrente (Python)"]
        Meta -->|"Orquestra lote"| Down["StreamDownloader\n(ThreadPoolExecutor)"]
        Down -->|"HTTP Range: bytes=N-"| Chunks["Streaming de Chunks (1 MB)"]
        Chunks -->|"Gravação contínua"| Part[".part (Arquivo Temporário)"]
        Part -->|"Tamanho validado via HEAD"| Zip[".zip (Download Concluído)"]
        Zip -->|"Descompactação"| CSV["CSV Bruto em Disco"]
    end

    subgraph S3["3. Processamento Zero-Copy e FFI (Rust)"]
        CSV -->|"to_parquet() | py.allow_threads"| Mmap["MmapReader (memmap2)\nMemória Virtual do Kernel"]
        Mmap -->|"find_chunk_boundaries"| Slices["Fatias &[u8] alinhadas por quebra de linha"]
        
        Slices -->|"Rayon::par_iter()"| W1["Worker Thread 1\n(clean_quotes + tipos)"]
        Slices -->|"Rayon::par_iter()"| W2["Worker Thread 2\n(clean_quotes + tipos)"]
        Slices -->|"Rayon::par_iter()"| Wn["Worker Thread N\n(clean_quotes + tipos)"]

        W1 -->|"RecordBatch"| Batches["Batches Colunares (Apache Arrow)"]
        W2 -->|"RecordBatch"| Batches
        Wn -->|"RecordBatch"| Batches
    end

    subgraph S4["4. Destino Analítico"]
        Batches -->|"ArrowWriter\nCompressão Snappy"| Parquet[".parquet (Arquivo Colunar Final)"]
        Parquet -.->|"Consulta sem banco"| OLAP["DuckDB / Polars / ClickHouse"]
    end
```

---

## 2. Diagrama de Sequência e Fronteiras FFI

O fluxo temporal demonstra as chamadas de método, a liberação explícita da GIL (*Global Interpreter Lock*) e a coordenação entre a orquestração Python e a extensão nativa:

```mermaid
sequenceDiagram
    autonumber

    box rgba(55, 118, 171, 0.15) Camada Python (Orquestração)
        actor User as Pipeline / Usuário
        participant Crawler as WebDavCrawler
        participant Downloader as StreamDownloader
        participant FFI as Wrapper Python (to_parquet)
    end

    box rgba(222, 165, 132, 0.15) Camada Nativa Rust (_core)
        participant Engine as lib.rs (PyO3)
        participant Mmap as MmapReader (reader.rs)
        participant Rayon as Pool Rayon (converter.rs)
    end

    box rgba(158, 206, 106, 0.15) Subsistema de Arquivos & Rede
        participant Remote as Servidor RFB
        participant Storage as Sistema de Arquivos
    end

    %% FASE 1: DESCOBERTA
    Note over User, Remote: Fase 1 — Descoberta e Catalogação
    User->>Crawler: catalog(url)
    Crawler->>Remote: HTTP PROPFIND /
    Remote-->>Crawler: XML multistatus
    Crawler-->>User: Metadados dos arquivos disponíveis

    %% FASE 2: DOWNLOAD CONCORRENTE
    Note over User, Storage: Fase 2 — Ingestão Resumable e Streaming
    User->>Downloader: download_files_concurrently()
    Downloader->>Remote: HTTP HEAD (verifica tamanho)
    Downloader->>Remote: HTTP GET (Range: bytes=N-)
    Remote-->>Downloader: Chunks de 1 MB
    Downloader->>Storage: Grava .part e converte em .zip/.csv

    %% FASE 3: CONVERSÃO NATIVA
    Note over User, Storage: Fase 3 — Conversão Nativa sem bloqueio de GIL
    User->>FFI: to_parquet(csv_path, parquet_path)
    FFI->>Engine: _core.to_parquet(...)
    Note over Engine: py.allow_threads (GIL liberado)

    Engine->>Mmap: from_path(csv_path)
    Mmap->>Storage: mmap (mapeia páginas de memória)
    Mmap-->>Engine: Fatias brutas &[u8] alinhadas por quebra de linha

    Engine->>Rayon: Processamento concorrente das fatias
    par Execução Concorrente em N Threads
        Rayon->>Rayon: Parsing zero-copy + Coerção de tipos
        Rayon->>Rayon: Construção de RecordBatches (Arrow)
    end

    Engine->>Storage: ArrowWriter (serialização Snappy para .parquet)
    Note over Engine: Reassociação do GIL
    Engine-->>FFI: total_linhas
    FFI-->>User: Retorno com métricas de vazão (linhas/s, MB/s)
```

---

## 3. Estágios Detalhados do Processamento

### Estágio 1: Descoberta e Catalogação
- Envio de requisição HTTP `PROPFIND` com profundidade 1.
- Extração de metadados (`displayname`, `getcontentlength`, `getlastmodified`) tolerante a variações de namespaces XML (`d:`, `oc:`, `nc:`).
- Mapeamento de prefixos nominais para os tipos de tabelas canônicas suportadas (`empresas`, `socios`, `estabelecimentos`, `simples`, etc.).

### Estágio 2: Download em Streaming e Resume
- Verificação se o arquivo local já existe com tamanho idêntico.
- Caso exista um arquivo `.part` parcial, envia cabeçalho `Range: bytes=<tam>-` para evitar re-download de dados já trafegados.
- Escrita atômica em blocos de 1 MB; ao finalizar com sucesso, renomeia `.part` para o nome definitivo (`.zip`).

### Estágio 3: FFI e Liberação de GIL
- `cnpydge.to_parquet(...)` valida caminhos e existência do arquivo fonte no Python.
- Chamada do binding Rust que executa `py.allow_threads(...)`, liberando a thread do Python para o loop de eventos ou outras tarefas.

### Estágio 4: Mapeamento de Memória Virtual (`mmap`)
- O arquivo é mapeado em memória virtual sem cópia intermediária de buffer via `memmap2`.
- As quebras de linha `\n` são identificadas de forma distribuída para calcular fatias contíguas (`&[u8]`) independentes.

### Estágio 5: Parsing Paralelo Zero-Copy
- Rayon distribui os blocos de bytes entre os núcleos da CPU.
- Parsers limpam aspas (`clean_quotes`), validam formato alfanumérico e convertem números/datas sem alocações de strings desnecessárias.
- Cada thread produz um `RecordBatch` Arrow.

### Estágio 6: Serialização Colunar Apache Parquet
- Um `ArrowWriter` consolidado recebe os `RecordBatch` gerados.
- Os dados são comprimidos com algoritmo Snappy e gravados de forma contínua em disco.
- Retorno da contagem de linhas e cálculo de vazão em megabytes por segundo (`MB/s`) e linhas por segundo (`linhas/s`).
