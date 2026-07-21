# ADR 0004: Vertical Slice Architecture para camada Python

## Status
Aceito

## Contexto
A camada Python de orquestracao cresce naturalmente com features novas (updater, company, search, export, metrics, benchmark, API, CLI, dashboard). Sem organizacao clara, o codigo tende a virar um monolito com acoplamento alto.

## Decisao
Adotar **Vertical Slice Architecture**: cada funcionalidade e um slice vertical com limites claros entre dominio e infraestrutura.

```
python/
├── app/               # inicializacao e composicao dos servicos
├── features/          # funcionalidades por dominio (vertical slice)
│   ├── updater/       # download e atualizacao da base RFB
│   ├── company/       # consulta e manipulacao de CNPJ
│   ├── search/        # mecanismos de pesquisa
│   ├── export/        # exportacao (CSV, Parquet, DuckDB, Postgres)
│   ├── metrics/       # metricas e monitoramento
│   └── benchmark/     # benchmarks de desempenho
├── shared/            # codigo compartilhado entre features
├── integrations/      # adaptadores para servicos externos
│   ├── duckdb/
│   ├── postgres/
│   ├── receita/
│   └── storage/
├── config/            # configuracao central (settings, logging, paths)
├── entrypoints/       # interfaces de entrada (API, CLI, Dashboard)
└── tests/             # testes unitarios e integracao
```

## Consequencias
- Cada feature pode evoluir independentemente sem quebrar outras.
- Sepacao clara: dominio (features/) vs infraestrutura (integrations/) vs interface (entrypoints/).
- Facil de testar: cada slice tem seus proprios testes.
- Novos destinos de dados entram como adaptadores em `integrations/`, nao como if/else hardcoded.

## Alternativas consideradas
- **Organizacao por camada (controllers/services/repositories)**: descartado por criar acoplamento horizontal -- mudanca em uma feature toca multiplas camadas.
- **Organizacao por tipo (models/utils/services)**: descartado por misturar dominios e dificultar localizacao de codigo relacionado.
