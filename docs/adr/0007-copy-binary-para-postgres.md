# ADR 0007: COPY BINARY para persistencia no Postgres

## Status
Aceito

## Contexto
A persistencia de centenas de milhoes de registros no Postgres precisa ser o mais rapida possivel. INSERT tradicional (linha a linha) e proibido pelo principio de performance.

## Decisao
Usar o **protocolo COPY BINARY** do Postgres para escrita em lote:

- O batch builder acumula registros (default: 100k) e envia como um unico COPY BINARY.
- Indices sao desabilitados antes do COPY e recriados depois.
- A escrita usa o formato binario do PostgreSQL, nao texto CSV.

## Consequencias
- Throughput de escrita 10-50x maior que INSERT tradicional.
- RAM constante: o batch e enviado e liberado imediatamente.
- Indices sao recriados apenas uma vez no final, nao a cada lote.

## Alternativas consideradas
- **INSERT em lote (batch)**: descartado por ser 10-50x mais lento que COPY BINARY.
- **INSERT com prepared statements**: descartado por ainda ser mais lento que COPY e por nao aproveitar o protocolo otimizado do Postgres.
- **Usar ORM (SQLAlchemy)**: descartado por introduzir overhead desnecessario no hot path.
