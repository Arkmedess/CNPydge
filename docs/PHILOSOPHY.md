# CNPJ Ultra Importer

> **Missao:** Construir o pipeline de ingestao da base publica do CNPJ da Receita Federal mais rapido, modular e eficiente possivel, utilizando Python para orquestracao e Rust para o caminho critico (hot path), com arquitetura preparada para processar centenas de milhoes de registros consumindo poucos megabytes de RAM.

---

# Filosofia

Este projeto nao e apenas um importador de dados.

E um estudo de engenharia de software de alta performance aplicado a maior base publica de empresas do Brasil.

Os principios sao:

* **Zero desperdicio de memoria**
* **Zero copias desnecessarias**
* **Streaming do inicio ao fim**
* **Escalabilidade horizontal**
* **Paralelismo em todas as etapas**
* **Arquitetura modular**
* **Baixo acoplamento**
* **Pipeline observavel**
* **Preparado para bilhoes de registros**

O limite de desempenho deve ser o hardware (SSD, CPU e banco de dados), e nao a linguagem utilizada.

(Documento completo original preservado como fornecido pela equipe -- ver historico do repositorio
para a versao integral com arquitetura ASCII, stack tecnologica, estrutura do projeto, pipeline
detalhado por etapa, roadmap tecnico e principios de engenharia numerados 1-10.)

Ver tambem: `.mimocode/skills/core/engineering-principles/SKILL.md` (versao operacionalizada
destes principios, usada pelo agente no dia a dia) e `.mimocode/skills/core/planning/SKILL.md`
(roadmap tecnico Fase 1-9).
