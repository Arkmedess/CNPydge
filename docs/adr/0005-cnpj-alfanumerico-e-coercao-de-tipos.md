# 5. Suporte a CNPJ Alfanumérico e Coerção Estrita de Tipos

Data: 2026-09-19
Status: Aceito

## Contexto
O ecossistema de dados da Receita Federal passa por duas realidades técnicas:
1. **Novo CNPJ Alfanumérico:** A transição regulatória da RFB estabeleceu que novos registros de CNPJ utilizam caracteres alfanuméricos (`0-9`, `A-Z`) mantendo a extensão de 14 posições (8 base, 4 ordem, 2 dígito verificador). Pipelines que assumem apenas números inteiros quebram com a nova especificação.
2. **Representação Bruta em CSV:** Os dados brutos utilizam convenções brasileiras e formatos legados que degradam performance analítica se mantidos como texto puro: capital social formatado com vírgula decimal (`"100000,00"`), datas textuais no formato `YYYYMMDD` e códigos de tabelas auxiliares que são estritamente inteiros armazenados entre aspas.

## Decisão
Implementar no motor nativo em Rust (`engine/src/util.rs`, `empresa.rs`, `socio.rs`, `estabelecimento.rs`) as seguintes regras de validação e coerção:
1. **Validação Alfanumérica Extensiva:** O `cnpj_basico` (8 bytes), `cnpj_ordem` (4 bytes) e `cnpj_dv` (2 bytes) são validados via `b.is_ascii_alphanumeric()` e preservados como `Utf8` no Apache Arrow/Parquet, assegurando compatibilidade com o formato alfanumérico presente e futuro.
2. **Parsing Monetário Brasileiro (`parse_capital`):** Leitura de valores monetários sem alocação, convertendo a representação brasileira com vírgula para ponto e gerando tipo primitivo `Float64`.
3. **Representação Temporal Compacta:** Campos de data no formato `YYYYMMDD` são convertidos em inteiros `UInt32` (ou nulos se vazios/inválidos), viabilizando filtros por faixa temporal e particionamento nativo no Parquet com footprint mínimo.
4. **Tipagem Numérica Compacta:** Códigos de natureza jurídica, CNAE, município, país e situação cadastral são convertidos em inteiros sem sinal (`UInt8`, `UInt16`, `UInt32`), eliminando o overhead de armazenamento e comparação de strings.

## Consequências
- **Ganhos:**
  - Compatibilidade imediata com a nova regulamentação de CNPJs alfanuméricos da Receita Federal.
  - Redução massiva no consumo de memória e no tamanho dos arquivos Parquet gerados.
  - Consultas analíticas em DuckDB/Polars muito mais velozes em agregações numéricas e filtros de data.
- **Compromissos:**
  - Rejeição estrita de linhas corrompidas que violem o tamanho ou caracteres esperados.
  - Clientes SQL que esperem tipos puramente `DATE` necessitam converter o inteiro `YYYYMMDD` ou consultar o campo diretamente como número ordenável.

