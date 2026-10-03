# 6. Ingestão por Esteira Unitária (Rolling Eviction) e Perfis de Armazenamento

Data: 2026-09-29
Status: Aceito

## Contexto
O download e processamento integral da base de CNPJ da Receita Federal gera um volume massivo de dados (10 GB em `.zip` e mais de 60 GB em `.csv` descompactados).
Extrair todos os arquivos de uma vez exige grande capacidade de disco livre (~70 GB a 100 GB). Por outro lado, a descompressão em streaming direto na memória (Zero-Disk DEFLATE) eliminaria a vantagem do leitor `memmap2` em paralelo via Rayon e introduziria um gargalo de CPU em thread única com alta complexidade técnica.

## Decisão
Adotar a **Esteira Unitária com Descarte Imediato (*Rolling Eviction*)** como estratégia padrão (`Profile.BALANCED`) para a orquestração do pipeline:
1. **Processamento Unitário Particionado:** Baixar 1 partição `.zip`, extrair o `.csv` correspondente, apagar imediatamente o `.zip`, converter o `.csv` via `mmap` em Rust para Apache Parquet e apagar imediatamente o `.csv`.
2. **Manifesto de Estado (Checkpoint):** Registrar transições atômicas de cada partição (`DOWNLOADED`, `EXTRACTED`, `CONVERTED`) para permitir retomada segura sem reprocessamento caso o pipeline seja interrompido.
3. **Perfis Configuráveis:**
   - `Profile.BALANCED` (Padrão): Esteira unitária com descarte imediato (pico de ~2 a 3 GB de disco temporário).
   - `Profile.PERFORMANCE`: Processamento paralelo em lote para ambientes com alto armazenamento SSD disponível.
   - `Profile.ARCHIVAL`: Converte e preserva os arquivos `.zip` originais para fins de auditoria.
4. **Streaming DEFLATE (Zero-Disk):** Registrado como alternativa postergada para pesquisa e teste futuro em ambientes serverless/efêmeros com restrição severa de I/O.

## Consequências
- **Ganhos:**
  - Redução drástica da exigência de disco temporário (de ~70 GB para ~3 GB no pico).
  - Preservação da máxima vazão do motor Rust nativo (`memmap2` particionado em paralelo via Rayon).
  - Tolerância a falhas garantida pelo manifesto de estado.
- **Compromissos:**
  - Em caso de falha durante a conversão do CSV com o ZIP já expurgado, o arquivo precisará ser rebaixado da origem caso não haja cópia em cache.
