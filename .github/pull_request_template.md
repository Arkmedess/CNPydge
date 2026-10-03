<!-- 
TEMPLATE PADRONIZADO DE PULL REQUEST — CNPydge
-->

## 1. Resumo das Alterações & Valor de Negócio
- **Qual problema/dor isso resolve?** 
- **O que foi implementado/alterado?** 
- **Impacto e benefícios operacionais:** 

---

## 2. Decisões Arquiteturais & Contratos
- [ ] Houve alteração em contratos públicos (`cnpydge/__init__.py`, stubs `_core.pyi` ou esquemas Parquet)?
- [ ] Há novo Registro de Decisão Arquitetural formalizado (`docs/adr/00XX-...`)?
- [ ] A documentação correspondente (`README.md`, docs locais de módulo) foi atualizada?

---

## 3. Garantias de Qualidade & Testes
- [ ] **Testes em Rust:** `cargo test --all-targets` executado e aprovado.
- [ ] **Linters Rust:** `cargo clippy --all-targets -- -D warnings` e `cargo fmt --all --check` sem alertas.
- [ ] **Testes em Python:** `uv run pytest --cov=cnpydge` com cobertura mínima de 80% (cobertura atual: `__%`).
- [ ] **Linters Python:** `uv run ruff check .` sem alertas.
- [ ] **Build Nativo:** `uv run --with maturin maturin develop` executado com sucesso.

---

## 4. Rastreabilidade & Commits
- **Commits atômicos incluídos:**
  - `<hash>` - `<tipo>(<escopo>): <descrição>`
- **Issues vinculadas:**
  - Closes #...
  - Refs #...
