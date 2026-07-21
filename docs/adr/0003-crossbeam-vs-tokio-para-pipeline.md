# ADR 0003: crossbeam vs tokio para pipeline concorrente

## Status
Aceito

## Contexto
O pipeline tem etapas sequenciais (parse -> normalizar -> batch -> persistir) que precisam rodar em paralelo com backpressure. A decisao e qual modelo de concorrencia usar.

## Decisao
Usar **crossbeam-channel** para o pipeline lock-free com topologia de threads dedicadas.

Cada etapa roda em sua propria thread, comunicando via canais bounded (backpressure natural). O batch builder acumula registros e envia lotes quando atinge o limiar configuravel (default: 100k registros).

## Consequencias
- Backpressure automatico: se o destino (Postgres) esta lento, o parser pausa naturalmente.
- Sem overhead de async runtime -- threads sao leves e previsiveis para workload CPU-bound.
- Topologia fixa e rastreavel: `Parser -> Normalizer -> BatchBuilder -> Destination`.
- Facil de instrumentar: cada canal pode expor metricas de throughput e latencia.

## Alternativas consideradas
- **tokio (async)**: descartado por ser overkill para pipeline CPU-bound com I/O sequencial. Async brilha em I/O concorrente (servidores HTTP), nao em ETL sequencial.
- **rayon (work-stealing)**: descartado por nao modelar bem pipeline com etapas dependentes -- rayon e ideal para paralelismo de dados (map/reduce), nao para pipeline com backpressure.
- **std::sync::mpsc**: descartado por nao suportar bounded channels com backpressure eficiente e por ter API menos ergonomica que crossbeam.
