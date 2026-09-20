# Catálogo de Esquemas Colunares (Apache Arrow & Parquet)

Este documento cataloga a especificação formal de esquemas utilizada pelo conversor nativo do **CNPydge** na geração dos arquivos Apache Parquet. A modelagem prioriza tipos primitivos compactos do Apache Arrow para otimizar espaço em disco e acelerar consultas analíticas (OLAP).

---

## 1. Tabelas Cadastrais Principais

### A. `Empresas`
Armazena a raiz cadastral de cada pessoa jurídica.

| Campo Arrow | Tipo Arrow | Nulável | Descrição Semântica |
| :--- | :--- | :---: | :--- |
| `cnpj_basico` | `Utf8` | Não | 8 dígitos ou caracteres alfanuméricos do CNPJ básico |
| `razao_social` | `Utf8` | Não | Razão social ou denominação empresarial completa |
| `natureza_juridica` | `UInt16` | Não | Código da natureza jurídica da entidade |
| `qualificacao_responsavel` | `UInt16` | Não | Código da qualificação da pessoa física responsável |
| `capital_social` | `Float64` | Não | Valor monetário em reais (convertido de `"100000,00"`) |
| `porte` | `UInt8` | Não | Porte da empresa (`01` Não Informado, `03` EPP, `05` Demais) |
| `ente_federativo` | `Utf8` | Sim | Ente federativo responsável (para entidades públicas) |

---

### B. `Socios`
Quadro de Sócios e Administradores (QSA) vinculado ao CNPJ básico.

| Campo Arrow | Tipo Arrow | Nulável | Descrição Semântica |
| :--- | :--- | :---: | :--- |
| `cnpj_basico` | `Utf8` | Não | 8 caracteres do CNPJ básico |
| `identificador_socio` | `UInt8` | Não | Tipo de sócio (`1` Pessoa Jurídica, `2` Pessoa Física, `3` Estrangeiro) |
| `nome_socio` | `Utf8` | Não | Nome ou razão social do sócio |
| `cpf_cnpj_socio` | `Utf8` | Não | CPF (descaracterizado/mascarado) ou CNPJ do sócio |
| `qualificacao_socio` | `UInt16` | Não | Código de qualificação societária (sócio, administrador, etc.) |
| `data_entrada_sociedade` | `UInt32` | Não | Data de ingresso no formato numérico `YYYYMMDD` |
| `pais` | `UInt16` | Sim | Código do país de origem (para sócio domiciliado no exterior) |
| `representante_legal` | `Utf8` | Sim | CPF descaracterizado do representante legal |
| `nome_representante` | `Utf8` | Sim | Nome do representante legal |
| `qualificacao_representante` | `UInt16` | Sim | Código de qualificação do representante legal |
| `faixa_etaria` | `UInt8` | Não | Código da faixa etária (`1` a `9`, onde `1` é 0-12 anos até `9` >80 anos) |

---

### C. `Estabelecimentos`
Dados de unidades (matriz e filiais), localização geográfica, CNAEs e contatos.

| Campo Arrow | Tipo Arrow | Nulável | Descrição Semântica |
| :--- | :--- | :---: | :--- |
| `cnpj_basico` | `Utf8` | Não | 8 caracteres do CNPJ básico |
| `cnpj_ordem` | `Utf8` | Não | 4 caracteres da ordem do estabelecimento (ex.: `0001`) |
| `cnpj_dv` | `Utf8` | Não | 2 caracteres do dígito verificador (ex.: `95`) |
| `matriz_filial` | `UInt8` | Não | Indicador de tipo (`1` Matriz, `2` Filial) |
| `nome_fantasia` | `Utf8` | Sim | Nome fantasia do estabelecimento |
| `situacao_cadastral` | `UInt8` | Não | Código de situação (`01` Nula, `02` Ativa, `03` Suspensa, `04` Inapta, `08` Baixada) |
| `data_situacao_cadastral`| `UInt32` | Sim | Data do evento cadastral no formato `YYYYMMDD` |
| `motivo_situacao_cadastral`| `UInt16` | Não | Código de motivo da situação cadastral |
| `nome_cidade_exterior` | `Utf8` | Sim | Nome da cidade no exterior (se aplicável) |
| `pais` | `UInt16` | Sim | Código do país no exterior |
| `data_inicio_atividade` | `UInt32` | Sim | Data de início das atividades (`YYYYMMDD`) |
| `cnae_fiscal_principal` | `UInt32` | Não | Código numérico da atividade econômica principal |
| `cnae_fiscal_secundaria`| `Utf8` | Sim | Lista de códigos CNAE secundários separados por vírgula |
| `tipo_logradouro` | `Utf8` | Sim | Tipo de logradouro (Rua, Avenida, etc.) |
| `logradouro` | `Utf8` | Sim | Nome do logradouro |
| `numero` | `Utf8` | Sim | Número do endereço |
| `complemento` | `Utf8` | Sim | Complemento do endereço |
| `bairro` | `Utf8` | Sim | Bairro ou distrito |
| `cep` | `Utf8` | Sim | Código de Endereçamento Postal (8 caracteres) |
| `uf` | `Utf8` | Sim | Sigla da Unidade Federativa |
| `municipio` | `UInt16` | Sim | Código SIAFI/RFB do município |
| `ddd_1` | `Utf8` | Sim | DDD do telefone primário |
| `telefone_1` | `Utf8` | Sim | Número do telefone primário |
| `ddd_2` | `Utf8` | Sim | DDD do telefone secundário |
| `telefone_2` | `Utf8` | Sim | Número do telefone secundário |
| `ddd_fax` | `Utf8` | Sim | DDD do fax |
| `fax` | `Utf8` | Sim | Número do fax |
| `correio_eletronico` | `Utf8` | Sim | E-mail de contato cadastrado |
| `situacao_especial` | `Utf8` | Sim | Situação especial (recuperação judicial, falência) |
| `data_situacao_especial`| `UInt32` | Sim | Data da ocorrência da situação especial (`YYYYMMDD`) |

---

### D. `Simples` (Simples Nacional / MEI)
Histórico e enquadramento nos regimes simplificados.

| Campo Arrow | Tipo Arrow | Nulável | Descrição Semântica |
| :--- | :--- | :---: | :--- |
| `cnpj_basico` | `Utf8` | Não | 8 caracteres do CNPJ básico |
| `opcao_simples` | `Utf8` | Sim | Indicador de opção (`S` Sim, `N` Não, `O` Outros) |
| `data_opcao_simples` | `UInt32` | Sim | Data de início da opção no formato `YYYYMMDD` |
| `data_exclusao_simples` | `UInt32` | Sim | Data de exclusão da opção no formato `YYYYMMDD` |
| `opcao_mei` | `Utf8` | Sim | Indicador de opção pelo MEI (`S` / `N`) |
| `data_opcao_mei` | `UInt32` | Sim | Data de início no MEI no formato `YYYYMMDD` |
| `data_exclusao_mei` | `UInt32` | Sim | Data de desenquadramento MEI no formato `YYYYMMDD` |

---

## 2. Tabelas Auxiliares de Domínio

### A. Tabela `Cnaes`
| Campo Arrow | Tipo Arrow | Nulável | Descrição Semântica |
| :--- | :--- | :---: | :--- |
| `codigo` | `UInt32` | Não | Código numérico da atividade econômica (7 posições) |
| `descricao` | `Utf8` | Não | Descrição textual oficial da classe/subclasse CNAE |

### B. Tabelas de Domínio Padronizadas com Código `UInt16`
As seguintes tabelas compartilham o mesmo esquema colunar uniforme:
- **`Motivos`:** Códigos de motivos de situação cadastral.
- **`Municipios`:** Códigos de identificação municipal da Receita Federal / SIAFI.
- **`Naturezas`:** Códigos de naturezas jurídicas do CONCLA/IBGE.
- **`Paises`:** Códigos de países segundo a tabela da Receita Federal.
- **`Qualificacoes`:** Códigos de qualificação dos integrantes societários ou representantes.

| Campo Arrow | Tipo Arrow | Nulável | Descrição Semântica |
| :--- | :--- | :---: | :--- |
| `codigo` | `UInt16` | Não | Código identificador da entidade de domínio |
| `descricao` | `Utf8` | Não | Descrição textual oficial correspondente |

