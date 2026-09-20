# 4. Coleta Concorrente e Download Resumable via HTTP Streaming

Data: 2026-09-20
Status: Aceito

## Contexto
A base da Receita Federal é distribuída mensalmente em dezenas de arquivos compactados (`.zip`), hospedados em diretórios com interface WebDAV. O processo de extração enfrenta dois desafios operacionais críticos:
1. Instabilidade de rede e conexões longas que frequentemente sofrem quedas e timeouts, o que invalidaria downloads parciais de arquivos de múltiplos gigabytes.
2. Lentidão excessiva na transferência quando os downloads são executados de forma puramente sequencial.

## Decisão
Implementar uma esteira de ingestão de rede no pacote Python (`orchestrator/cnpydge/`) baseada em:
1. **Catalogação WebDAV:** Uso de requisições `PROPFIND` com parsing XML resiliente aos namespaces (`DAV:`, `owncloud`, `nextcloud`) para identificar arquivos e seus tamanhos (`Content-Length`).
2. **Download em Streaming Atômico:** Gravação em arquivos temporários `.part` via streaming em blocos (`chunk_size` padrão de 1 MB) através do `httpx`, evitando alocação dos arquivos na memória principal.
3. **Retomada de Download (Resume):** Verificação de integridade preliminar via `HEAD` e utilização do cabeçalho HTTP `Range: bytes=<offset>-` para retomar downloads interrompidos a partir do último byte gravado.
4. **Paralelismo Controlado:** Orquestração de múltiplos downloads simultâneos via `ThreadPoolExecutor` com controle de concorrência para maximizar o throughput da banda sem saturar o servidor de origem.

## Consequências
- **Ganhos:**
  - Tolerância a falhas de rede com eliminação de re-download de gigabytes já trafegados.
  - Garantia de atomicidade: o arquivo só é renomeado para o destino final se o tamanho final bater com o cabeçalho remoto.
  - Consumo de RAM previsível e restrito ao buffer de 1 MB por thread.
- **Compromissos:**
  - Dependência do suporte do servidor de origem ao cabeçalho `Range` (com fallback graceful para download integral caso não suporte).
  - Necessidade de gerenciar concorrência responsável para não incorrer em *rate-limiting* no endpoint governamental.

