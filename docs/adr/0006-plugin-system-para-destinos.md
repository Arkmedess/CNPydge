# ADR 0006: Plugin system para destinos de dados

## Status
Proposto (Fase 9 do roadmap)

## Contexto
O pipeline precisa suportar multiplos destinos de dados (Postgres, DuckDB, ClickHouse, Elasticsearch, SQLite, Parquet, S3, BigQuery). Sem uma abordagem modular, cada novo destino vira um if/else hardcoded no pipeline.

## Decisao
Na Fase 9, implementar um **plugin system** baseado em traits/interfaces Rust:

- Cada destino implementa um trait `Destination` com metodos `write_batch()`, `flush()`, `close()`.
- O pipeline aceita qualquer implementacao do trait, sem conhecer o destino concreto.
- Novos destinos sao adicionados como crates independentes ou modulos separados.

## Consequencias
- Adicionar um novo destino nao requer mudanca no parser ou no pipeline.
- Cada destino pode ter sua propria estrategia de batching e persistencia.
- Testes de integracao podem mockar o trait `Destination`.

## Alternativas consideradas
- **if/else hardcoded**: descartado por violar o principio de baixo acoplamento e por tornar o pipeline rigido.
- **Config-driven (YAML/JSON)**: considerado como complemento, nao substituto -- a logica de cada destino precisa de codigo Rust, nao so configuracao.
