# 2. Ingestão via Mapeamento de Memória (mmap) e Parsing Zero-Copy com Liberação de GIL

Data: 2026-09-18
Status: Aceito

## Contexto
Os arquivos brutos descompactados da Receita Federal (ex.: `Estabelecimentos` e `Empresas`) ultrapassam múltiplos gigabytes individualmente (chegando a dezenas de GBs quando somados). A leitura convencional baseada em buffers tradicionais (`BufferedReader` ou `read_to_string`) acarreta:
1. Cópias intermediárias contínuas de memória entre o espaço de kernel e o espaço de usuário.
2. Alto risco de estouro de memória (Out-Of-Memory) em ambientes com restrição de RAM.
3. Gargalo de concorrência caso o runtime do Python mantenha o GIL (*Global Interpreter Lock*) ativo durante o parsing.

## Decisão
Implementar a ingestão dos arquivos utilizando `memmap2::Mmap` e fatiamento sem cópia (*zero-copy*):
1. **Mapeamento de Memória Virtual:** O arquivo é mapeado integralmente via `Mmap::map`, delegando a paginação e o gerenciamento de cache sob demanda ao subsistema de memória virtual do kernel.
2. **Particionamento Alinhado a Linhas:** O algoritmo `find_chunk_boundaries` calcula os deslocamentos `(offset_inicio, offset_fim)` buscando o byte de quebra de linha (`\n`) mais próximo, permitindo que fatias contíguas (`&[u8]`) sejam processadas de forma independente e sem realocação.
3. **Liberação de GIL:** Toda a rotina de parsing e conversão nativa é encapsulada em `py.allow_threads(|| ...)` no ponto de entrada do PyO3, liberando o interpretador Python para executar outras tarefas concorrentes.

## Consequências
- **Ganhos:**
  - Consumo de memória RAM constante e previsível, independente do tamanho do arquivo CSV de entrada.
  - Zero alocações de heap durante a determinação de limites de blocos e leitura de fatias brutas.
  - Paralelismo real em nível de sistema operacional enquanto a conversão ocorre no motor Rust.
- **Compromissos:**
  - O uso de `unsafe` em `memmap2::Mmap` exige a invariante de que os arquivos de dados permaneçam imutáveis no disco durante o processamento.
  - O I/O em discos magnéticos lentos (HDDs) com `mmap` pode gerar latências em caso de faltas de página (*page faults*) dispersas, sendo otimizado primariamente para leitura sequencial e armazenamento SSD/NVMe.

