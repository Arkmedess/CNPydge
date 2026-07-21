# ADR 0002: mmap vs Buffered Read para parsing de CSV

## Status
Aceito

## Contexto
Os arquivos da RFB sao CSVs gigantes (dezenas a centenas de GB). A decisao e como alimentar o parser: carregar trechos em buffer ou mapear o arquivo inteiro na memoria virtual?

## Decisao
Usar **memory-mapped files (mmap)** via crate `memmap2` para leitura dos arquivos CSV.

O parser recebe um slice `&[u8]` mapeado e processa direto, sem copias intermediarias. O SO gerencia page faults e cache do disco, mantendo RAM constante independente do tamanho do arquivo.

## Consequencias
- RAM permanece ~constante: 500MB ou 500GB de arquivo, mesmo footprint em RAM.
- Throughput limitado pela velocidade do disco (SSD NVMe: ~3 GB/s), nao por alocacoes.
- Parsing pode operar com zero-copy sobre os bytes mapeados.
- Funciona em Linux/macOS/Windows (memmap2 e cross-platform).

## Alternativas consideradas
- **BufReader (buffered read)**: descartado por exigir gerenciamento manual de buffers, copias de dados entre kernel e userspace, e nao aproveitar o page cache do SO de forma transparente.
- **Carregar arquivo inteiro em RAM**: descartado por violar o principio de memoria constante -- impossivel com arquivos de 100+ GB.
- **Streaming com tokio::io**: descartado por introduzir complexidade assincrona desnecessaria no parsing sequencial, e por nao oferecer vantagem real sobre mmap para I/O sequencial.
