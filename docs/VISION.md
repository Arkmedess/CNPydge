# CNPJ Ultra Importer -- Visao

> Uma plataforma de ingestao de alta performance para a base publica de CNPJ da Receita Federal.
> Desenvolvida com uma unica premissa: o limite deve ser o hardware, nunca o software.

A base publica de CNPJ da Receita Federal e um dos maiores conjuntos de dados abertos do Brasil.
Dezenas de arquivos compactados, centenas de milhoes de registros e centenas de gigabytes de
informacao que precisam ser importados, normalizados e disponibilizados para consulta.

O **CNPJ Ultra Importer** parte de um principio simples:

> Se o disco consegue entregar 3 GB/s, o software deve ser capaz de consumir 3 GB/s.

Principios: Streaming First, Zero Copy, Memoria Constante, Hardware-Oriented Design,
Modularidade, Observabilidade.

Dois dominios: Python (orquestracao -- download, config, CLI, API, dashboard) e Rust
(nucleo computacional -- leitura, parsing, normalizacao, vetorizacao, batching, persistencia).

(Documento completo original preservado como fornecido pela equipe -- ver historico do
repositorio para a versao integral, incluindo secoes de arquitetura conceitual, paralelismo,
escalabilidade, banco de dados, benchmarks, engenharia antes de frameworks, roadmap filosofico
e o manifesto final.)

Ver tambem: `.mimocode/skills/core/engineering-principles/SKILL.md` para a versao acionavel
destes principios.
