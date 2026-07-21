# Formato dos arquivos da Receita Federal

Notas sobre o leiaute oficial de cada arquivo disponibilizado pela RFB para a base publica de CNPJ.

## Arquivos principais

| Arquivo | Descricao | Formato |
|---|---|---|
| `Empresas*.csv` | Dados gerais da empresa (razao social, porte, natureza juridica) | CSV pipe-delimited |
| `Estabelecimentos*.csv` | Endereco, atividade, situacao cadastral | CSV pipe-delimited |
| `Socios*.csv` | Socios e participacoes | CSV pipe-delimited |
| `Simples*.csv` | Opcao pelo Simples Nacional | CSV pipe-delimited |
| `CNAEs*.csv` | Tabela de CNAEs (Classificacao Nacional de Atividades Economicas) | CSV pipe-delimited |
| `Naturezas*.csv` | Tabela de naturezas juridicas | CSV pipe-delimited |
| `Paises*.csv` | Tabela de paises | CSV pipe-delimited |
| `Municipios*.csv` | Tabela de municipios (IBGE) | CSV pipe-delimited |
| `Qualificacoes*.csv` | Tabela de qualificacoes dos socios | CSV pipe-delimited |

## Caracteristicas gerais

- **Delimitador**: pipe (`|`)
- **Encoding**: ISO-8859-1 (Latin-1)
- **Quebra de linha**: LF (Unix)
- **CNPJ**: formato alfanumerico (14 caracteres: 12 + 2 DV)
- **Campos vazios**: representados por campo vazio entre delimitadores (nao por NULL ou espaco)

## Formato CNPJ alfanumerico

A RFB mudou o formato do CNPJ de apenas numerico (14 digitos) para alfanumerico:

```
Antigo:  12345678000195  (14 digitos numericos)
Novo:    123456780001AB  (12 caracteres + 2 DV alfanumericos)
```

O algoritmo de verificacao foi adaptado para suportar caracteres A-Z nas posicoes 13-14.

Ver ADR-0005 em `docs/adr/0005-formato-cnpj-alfanumerico.md`.

## Downloads

Os arquivos estao disponiveis em: https://dadosabertos.rfb.gov.br/CNPJ/

Cada release mensal contem arquivos compactados (ZIP) com os dados atualizados.

## Notas de implementacao

- O parser Rust (`rust/cnpj_core/src/parser.rs`) le os bytes diretamente do mmap, sem conversao de encoding.
- A normalizacao (`rust/cnpj_core/src/normalize.rs`) preserva o formato original dos campos.
- O encoding ISO-8859-1 e tratado na camada de parsing, nao na normalizacao.
