# Guia de Desenvolvimento

Guia completo para desenvolvedores que vão contribuir com o CNPydge.

## Visão Geral da Arquitetura

O CNPydge separa responsabilidades em dois domínios:

- **Python (orquestração)**: CLI, config, download, dashboard, API, logs
- **Rust (hot path)**: mmap, parser SIMD, normalização zero-copy, pipeline lock-free, batch builder, COPY BINARY

Integração via **PyO3 + maturin**.

```
Python (orquestração)              Rust (cnpj_core via PyO3)
--------------------------------  ------------------------------------
CLI (Typer)                        mmap (memmap2)
Config (Pydantic)                  Parser CSV SIMD (csv-core/memchr/simdutf8)
Download (httpx)                   Normalização zero-copy
Dashboard (Rich)                   Pipeline lock-free (crossbeam)
API (FastAPI)                      Batch builder (100k registros)
Logs (loguru)                      COPY BINARY (Postgres) / DuckDB / Arrow
```

**Regra de ouro**: Se envolve iterar sobre registros do CSV, é Rust. Se envolve orquestrar, é Python.

## Pré-requisitos

### Sistema Operacional

- Linux (recomendado: Ubuntu 22.04+)
- macOS (via Homebrew)
- Windows (via WSL2)

### Ferramentas

| Ferramenta | Versão mínima | Instalação |
|---|---|---|
| Python | 3.14+ | https://www.python.org/downloads/ |
| Rust | stable | https://rustup.rs/ |
| uv | latest | `curl -LsSf https://astral.sh/uv/install.sh \| sh` |
| PostgreSQL | 14+ | https://www.postgresql.org/download/ |

### Verificar instalação

```bash
python --version   # Python 3.14+
rustc --version    # rustc 1.75+
uv --version       # uv 0.4+
cargo --version    # cargo 1.75+
```

## Setup do Ambiente

### 1. Clonar e inicializar

```bash
git clone https://github.com/seu-usuario/CNPydge.git
cd CNPydge
./scripts/bootstrap.sh
```

O script `bootstrap.sh`:

1. Verifica pré-requisitos (uv, cargo)
2. Sincroniza dependências Python (`uv sync`)
3. Builda a extensão Rust (`maturin develop --release`)
4. Roda testes Rust (`cargo test`)

### 2. Verificar instalação

```bash
# Testes Python
uv run pytest

# Testes Rust
cd rust/cnpj_core && cargo test && cd ../..

# CLI
uv run python -m python.entrypoints.cli --help
```

### 3. Configurar banco de dados

```bash
# Criar banco de dados
createdb cnpj

# Configurar DSN
export CNPJ_POSTGRES_DSN="postgresql://localhost:5432/cnpj"

# Ou criar config.toml
cat > config.toml << EOF
postgres_dsn = "postgresql://localhost:5432/cnpj"
batch_size = 100_000
download_dir = "./data/cache"
EOF
```

## Estrutura de Trabalho

### Diretórios principais

```
CNPydge/
├── rust/cnpj_core/src/    # Código Rust (hot path)
├── python/                # Código Python (orquestração)
├── docs/                  # Documentação
├── scripts/               # Scripts de automação
└── benchmarks/            # Datasets e benchmarks
```

### Convenções de nomenclatura

- **Python**: `snake_case` para variáveis e funções, `PascalCase` para classes
- **Rust**: `snake_case` para variáveis e funções, `PascalCase` para tipos
- **Arquivos**: `snake_case` para Python, `snake_case` para Rust
- **Branches**: `feat/nome-da-feature`, `fix/nome-do-fix`, `docs/nome-da-doc`

## Desenvolvimento Python

### Estrutura de um módulo

Cada feature segue o padrão Vertical Slice:

```
python/features/nome-feature/
├── __init__.py
├── dominio.py         # Lógica de negócio
├── adaptador.py       # Integração com externos
└── test_dominio.py    # Testes
```

### Exemplo: Criar nova feature

```bash
# 1. Criar estrutura
mkdir -p python/features/minha_feature
touch python/features/minha_feature/__init__.py
touch python/features/minha_feature/dominio.py
touch python/features/minha_feature/test_dominio.py

# 2. Implementar
cat > python/features/minha_feature/dominio.py << 'EOF'
"""Minha feature."""

def processar_dados(dados: list[dict]) -> list[dict]:
    """Processa dados de entrada."""
    return [transformar(d) for d in dados]

def transformar(dado: dict) -> dict:
    """Transforma um registro."""
    return {**dado, "processado": True}
EOF

# 3. Testar
cat > python/features/minha_feature/test_dominio.py << 'EOF'
"""Testes da feature."""
from python.features.minha_feature.dominio import processar_dados

def test_processar_dados():
    dados = [{"id": 1}, {"id": 2}]
    resultado = processar_dados(dados)
    assert len(resultado) == 2
    assert all(r["processado"] for r in resultado)
EOF

# 4. Rodar testes
uv run pytest python/features/minha_feature/test_dominio.py -v
```

### Configuração

O sistema usa Pydantic Settings com três fontes:

1. **Variáveis de ambiente** (prioridade máxima): `CNPJ_*`
2. **Arquivo TOML**: `config.toml`
3. **Arquivo `.env`**: `.env`

```python
from python.config.settings import get_settings

settings = get_settings()
print(settings.postgres_dsn)
print(settings.batch_size)
```

### Logging

Usamos `loguru` para logging estruturado:

```python
from loguru import logger

logger.info("Iniciando processamento...")
logger.warning("Arquivo não encontrado: {}", filename)
logger.error("Falha na conexão: {}", error)
```

## Desenvolvimento Rust

### Estrutura de um módulo

```
rust/cnpj_core/src/
├── lib.rs           # Interface PyO3
├── modulo.rs        # Lógica do módulo
└── modulo_test.rs   # Testes (opcional)
```

### Exemplo: Criar novo módulo

```bash
# 1. Criar arquivo
cat > rust/cnpj_core/src/meu_modulo.rs << 'EOF'
//! Meu módulo Rust.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MeuErro {
    #[error("Erro de validação: {0}")]
    Validacao(String),
}

pub fn processar_dados(dados: &[u8]) -> Result<Vec<u8>, MeuErro> {
    // Lógica de processamento
    if dados.is_empty() {
        return Err(MeuErro::Validacao("Dados vazios".into()));
    }
    Ok(dados.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processar_dados() {
        let dados = b"teste";
        let resultado = processar_dados(dados).unwrap();
        assert_eq!(resultado, dados);
    }
}
EOF

# 2. Adicionar ao lib.rs
echo "mod meu_modulo;" >> rust/cnpj_core/src/lib.rs

# 3. Testar
cd rust/cnpj_core && cargo test meu_modulo && cd ../..
```

### Regras do hot path

1. **Sem panic**: Use `Result` e `thiserror`, nunca `unwrap()` ou `panic!()`
2. **Zero-copy**: Trabalhe com `&[u8]`, não `String` ou `Vec`
3. **Sem alocação desnecessária**: Reutilize buffers quando possível
4. **Comentários SAFETY**: Todo código `unsafe` deve ter justificativa

### Exemplo: Função segura

```rust
/// Normaliza um campo CNPJ.
///
/// # Arguments
/// * `campo` - Bytes do campo CNPJ (14 caracteres)
///
/// # Returns
/// Array fixo de 14 bytes com o CNPJ normalizado.
///
/// # Errors
/// Retorna erro se o campo tiver tamanho inválido.
pub fn normalizar_cnpj(campo: &[u8]) -> Result<[u8; 14], NormalizacaoErro> {
    if campo.len() != 14 {
        return Err(NormalizacaoErro::TamanhoInvalido {
            esperado: 14,
            atual: campo.len(),
        });
    }

    let mut resultado = [0u8; 14];
    resultado.copy_from_slice(campo);
    Ok(resultado)
}
```

## Testes

### Python

```bash
# Todos os testes
uv run pytest

# Com verbose
uv run pytest -v --tb=long

# Testes específicos
uv run pytest python/tests/test_downloader.py

# Com cobertura
uv run pytest --cov=python --cov-report=html
```

### Rust

```bash
# Todos os testes
cd rust/cnpj_core && cargo test

# Com verbose
cd rust/cnpj_core && cargo test -- --nocapture

# Testes específicos
cd rust/cnpj_core && cargo test test_nome_do_teste

# Benchmarks
cd rust/cnpj_core && cargo bench
```

### Integração

```bash
# Testes de integração (requer PostgreSQL rodando)
uv run pytest python/tests/integration/ -v
```

## Benchmarks

### Rodar benchmarks

```bash
# Rust (Criterion)
cd rust/cnpj_core && cargo bench

# Python (pytest-benchmark)
uv run pytest benchmarks/ -v
```

### Adicionar novo benchmark

```rust
// rust/cnpj_core/benches/meu_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn meu_benchmark(c: &mut Criterion) {
    c.bench_function("processar_dados", |b| {
        b.iter(|| processar_dados(black_box(&[1, 2, 3, 4, 5])))
    });
}

criterion_group!(benches, meu_benchmark);
criterion_main!(benches);
```

## Debugging

### Python

```bash
# Com py-spy (profiling)
uv run py-spy top --pid <PID>

# Com debugger
uv run python -m pdb python/entrypoints/cli.py
```

### Rust

```bash
# Com gdb
cd rust/cnpj_core && cargo build && gdb target/debug/cnpj_core

# Com valgrind (memória)
cd rust/cnpj_core && cargo build && valgrind ./target/debug/cnpj_core
```

## Deploy

### Produção

```bash
# Build release
uv run maturin develop --release

# Rodar
uv run python -m python.entrypoints.cli importar
```

### Docker

```bash
# Build imagem
docker build -t cnpydge .

# Rodar
docker run -e CNPJ_POSTGRES_DSN="postgresql://..." cnpydge
```

## Troubleshooting

### Erro: "maturin not found"

```bash
uv sync --all-extras
```

### Erro: "cnpj_core not found"

```bash
uv run maturin develop --release
```

### Erro: "PostgreSQL connection refused"

Verifique se o PostgreSQL está rodando:

```bash
pg_isready
```

### Erro: "Memory limit exceeded"

Reduza o `batch_size`:

```bash
export CNPJ_BATCH_SIZE=50000
```

## Recursos Úteis

- [Documentação PyO3](https://pyo3.rs/)
- [Documentação maturin](https://www.maturin.rs/)
- [Crossbeam](https://docs.rs/crossbeam/)
- [memmap2](https://docs.rs/memmap2/)
- [Typer](https://typer.tiangolo.com/)
- [Pydantic Settings](https://docs.pydantic.dev/latest/concepts/pydantic_settings/)
