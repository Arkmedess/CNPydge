# ADR 0005: Tratamento do formato CNPJ alfanumerico

## Status
Aceito

## Contexto
A Receita Federal mudou o formato do CNPJ de apenas numerico (14 digitos) para alfanumerico (12 caracteres + 2 digitos de verificacao). Isso afeta parsing, normalizacao, validacao e persistencia.

## Decisao
- O parser Rust aceita CNPJ alfanumerico nativamente (14 caracteres no total).
- A normalizacao preserva o formato original sem conversao para numerico.
- A validacao de digito verificador usa algoritmo adaptado para caracteres alfanumericos.
- O banco de dados armazena como VARCHAR(14), nao NUMERIC.

## Consequencias
- Compatibilidade retroativa: CNPJs antigos (14 digitos numericos) continuam funcionando.
- O parser nao precisa de branching por tipo de CNPJ -- trata tudo como string.
- Indexacao por CNPJ usa VARCHAR, que e ligeiramente mais lenta que INTEGER, mas necessaria.

## Alternativas consideradas
- **Converter tudo para numerico (base 36)**: descartado por introduzir perda de informacao e complexidade de conversao desnecessaria.
- **Dois parsers (um para numerico, outro para alfanumerico)**: descartado por duplicar logica e violar o principio de simplicidade.
