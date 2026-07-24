# Casos de Uso

Exemplos práticos de como usar o CNPydge para diferentes cenários.

## 1. Importação Completa da Base CNPJ

### Cenário

Você precisa importar toda a base pública de CNPJ da Receita Federal para um banco PostgreSQL.

### Pré-requisitos

- PostgreSQL rodando com banco `cnpj` criado
- Conexão com a internet para download dos arquivos
- Espaço em disco: ~50GB para arquivos ZIP + ~100GB para dados descompactados

### Passos

```bash
# 1. Configurar conexão com o banco
export CNPJ_POSTGRES_DSN="postgresql://usuario:senha@localhost:5432/cnpj"

# 2. Executar importação completa
uv run python -m python.entrypoints.cli importar

# 3. Monitorar progresso
# O CLI mostra throughput, registros processados e erros em tempo real
```

### Resultado esperado

```
Iniciando importacao CNPJ...
PostgreSQL: localhost:5432/cnpj
Batch size: 100000
Baixando 36 arquivos...
36 arquivos baixados com sucesso.
Importando 36 arquivos CSV...
Importacao concluida!
  Registros processados: 250000000
  Bytes lidos: 45.2 GB
  Lotes enviados: 2500
  Registros gravados: 250000000
  Erros: 0
  Tempo: 180.5s
  Throughput: 1385027 registros/s
  Throughput: 250.5 MB/s
```

### Métricas de performance

- **Throughput**: ~1.4M registros/s (SSD NVMe)
- **RAM**: ~50MB constante (independente do tamanho do arquivo)
- **Tempo**: ~3 minutos para base completa

---

## 2. Importação Seletiva (Apenas Empresas)

### Cenário

Você precisa apenas dos dados de empresas, sem estabelecimentos ou sócios.

### Passos

```bash
# Importar apenas arquivos de empresas
uv run python -m python.entrypoints.cli importar \
  --files Empresas0.zip Empresas1.zip Empresas2.zip
```

### Resultado esperado

```
Importando 3 arquivos CSV...
Importacao concluida!
  Registros processados: 45000000
  Throughput: 1500000 registros/s
```

---

## 3. Verificação de Status do Cache

### Cenário

Você quer verificar quais arquivos já foram baixados e estão em cache.

### Passos

```bash
# Ver status do cache
uv run python -m python.entrypoints.cli status
```

### Resultado esperado

```
Arquivos em cache: 36
  CNAEs.zip (2.1 MB)
  Empresas0.zip (450.2 MB)
  Empresas1.zip (445.8 MB)
  ...
```

---

## 4. Download com Verificação de Integridade

### Cenário

Você quer garantir que os arquivos baixados não estão corrompidos.

### Passos

```python
from pathlib import Path
from python.features.updater.downloader import download_file, compute_sha256
import httpx

# Baixar arquivo com verificação SHA256
async with httpx.AsyncClient() as client:
    path = await download_file(
        client=client,
        filename="Empresas0.zip",
        dest_dir=Path("./data/cache"),
        expected_sha256="abc123..."  # Hash esperado
    )
    print(f"Arquivo baixado: {path}")
    print(f"SHA256: {compute_sha256(path)}")
```

---

## 5. Configuração Avançada

### Cenário

Você precisa de configurações específicas para seu ambiente.

### Opção 1: Arquivo config.toml

```toml
# config.toml
postgres_dsn = "postgresql://user:pass@host:5432/cnpj"
batch_size = 50000
max_workers = 8
download_dir = "/data/cnpj/cache"
log_level = "DEBUG"
log_file = "/var/log/cnpj/import.log"
api_host = "0.0.0.0"
api_port = 8080
```

### Opção 2: Variáveis de ambiente

```bash
export CNPJ_POSTGRES_DSN="postgresql://user:pass@host:5432/cnpj"
export CNPJ_BATCH_SIZE=50000
export CNPJ_MAX_WORKERS=8
export CNPJ_DOWNLOAD_DIR="/data/cnpj/cache"
export CNPJ_LOG_LEVEL="DEBUG"
```

### Opção 3: Arquivo .env

```bash
# .env
CNPJ_POSTGRES_DSN=postgresql://user:pass@host:5432/cnpj
CNPJ_BATCH_SIZE=50000
CNPJ_MAX_WORKERS=8
```

---

## 6. Uso da API REST

### Cenário

Você quer expor uma API para consultas sobre a base importada.

### Passos

```bash
# Iniciar servidor
uv run uvicorn python.entrypoints.api:app --reload --host 0.0.0.0 --port 8000
```

### Endpoints disponíveis

```bash
# Health check
curl http://localhost:8000/health

# Status da importação
curl http://localhost:8000/status

# Iniciar importação (async)
curl -X POST http://localhost:8000/import \
  -H "Content-Type: application/json" \
  -d '{"files": ["Empresas0.zip"]}'
```

---

## 7. Integração com DuckDB (Fase 4)

### Cenário

Você quer usar DuckDB para analytics local sobre a base CNPJ.

### Passos (quando implementado)

```python
import duckdb
from python.integrations.duckdb import DuckDBAdapter

# Criar adapter
adapter = DuckDBAdapter(database="./data/cnpj.duckdb")

# Importar dados
adapter.importar(dados)

# Consultar
resultado = adapter.query("""
    SELECT porte, COUNT(*) as total
    FROM empresas
    GROUP BY porte
    ORDER BY total DESC
""")
print(resultado)
```

---

## 8. Exportação para Parquet (Fase 3)

### Cenário

Você quer exportar a base para formato Parquet para uso com Spark ou outras ferramentas.

### Passos (quando implementado)

```python
from python.features.export import exportar_parquet

# Exportar para Parquet
exportar_parquet(
    dados=empresas,
    destino="./data/empresas.parquet",
    compression="snappy"
)
```

---

## 9. Dashboard de Monitoramento (Fase 6)

### Cenário

Você quer monitorar a importação em tempo real com uma interface visual.

### Passos (quando implementado)

```bash
# Iniciar dashboard
uv run python -m python.entrypoints.dashboard
```

### Funcionalidades

- Throughput em tempo real
- Uso de memória
- Progresso da importação
- Logs estruturados
- Gráficos de performance

---

## 10. Benchmark Comparativo

### Cenário

Você quer comparar a performance do CNPydge com outras soluções.

### Passos

```bash
# Rodar benchmarks
cd rust/cnpj_core && cargo bench

# Resultados em target/criterion/
```

### Métricas coletadas

- Throughput (registros/s)
- Uso de memória (MB)
- Latência por registro (μs)
- Comparação com baseline

---

## 11. Desenvolvimento de Nova Feature

### Cenário

Você quer adicionar uma nova funcionalidade ao projeto.

### Passos

```bash
# 1. Criar branch
git checkout -b feat/minha-feature

# 2. Criar estrutura
mkdir -p python/features/minha_feature
touch python/features/minha_feature/__init__.py
touch python/features/minha_feature/dominio.py
touch python/features/minha_feature/test_dominio.py

# 3. Implementar (seguindo Vertical Slice)
# ...

# 4. Testar
uv run pytest python/features/minha_feature/ -v

# 5. Documentar
# Criar README.md no módulo
# Criar ADR se necessário

# 6. Commitar
git add .
git commit -m "feat: adicionar minha feature"

# 7. Push e PR
git push origin feat/minha-feature
```

---

## 12. Debugging de Performance

### Cenário

A importação está lenta e você quer identificar o gargalo.

### Passos

```bash
# 1. Rodar com profiling
uv run py-spy top --pid $(pgrep -f "cnpj_core")

# 2. Verificar métricas do Rust
cd rust/cnpj_core && cargo bench

# 3. Analisar logs
tail -f /var/log/cnpj/import.log | grep -E "throughput|error"
```

### Ferramentas úteis

- **py-spy**: Profiling Python
- **perf**: Profiling Rust
- **valgrind**: Análise de memória
- **Criterion**: Benchmarks Rust

---

## 13. Deploy em Produção

### Cenário

Você quer colocar o CNPydge em produção para importação automatizada.

### Passos

```bash
# 1. Build release
uv run maturin develop --release

# 2. Configurar cron (importação diária)
# Adicionar ao crontab:
0 2 * * * cd /opt/cnpydge && uv run python -m python.entrypoints.cli importar

# 3. Monitorar
# Configurar alertas baseados nas métricas
```

### Configuração recomendada

- **Servidor**: 8+ cores, 32GB RAM, SSD NVMe
- **Banco**: PostgreSQL 14+ com 16+ conexões
- **Rede**: 100Mbps+ para download da RFB
- **Backup**: Backup diário do banco de dados

---

## 14. Contribuição para o Projeto

### Cenário

Você quer contribuir com código, documentação ou correções de bugs.

### Passos

```bash
# 1. Fork e clone
git clone https://github.com/seu-usuario/CNPydge.git

# 2. Setup do ambiente
./scripts/bootstrap.sh

# 3. Criar branch
git checkout -b fix/meu-fix

# 4. Implementar e testar
# ...

# 5. Commitar (Conventional Commits)
git commit -m "fix: corrigir parsing de CNPJ alfanumérico"

# 6. Push e PR
git push origin fix/meu-fix
# Abrir PR no GitHub
```

### Diretrizes

- Seguir `CONTRIBUTING.md`
- Usar Conventional Commits
- Adicionar testes
- Atualizar documentação
- Respeitar princípios de engenharia

---

## 15. Integração com CI/CD

### Cenário

Você quer configurar integração contínua para o projeto.

### Exemplo de workflow GitHub Actions

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      - name: Install uv
        uses: astral-sh/setup-uv@v3
      - name: Setup Python
        run: uv python install 3.14
      - name: Install dependencies
        run: uv sync
      - name: Build Rust
        run: uv run maturin develop --release
      - name: Run Rust tests
        run: cd rust/cnpj_core && cargo test
      - name: Run Python tests
        run: uv run pytest
      - name: Run linters
        run: |
          uv run ruff check python/
          uv run mypy python/
          cd rust/cnpj_core && cargo clippy -- -D warnings
```

---

## Casos de Uso Futuros (Fases 5-9)

### Atualização Incremental (Fase 5)

- Importar apenas diferenças entre releases mensais
- Reduzir tempo de atualização de horas para minutos

### Dashboard (Fase 6)

- Interface visual para monitoramento
- Gráficos de throughput e uso de memória
- Alertas de performance

### API REST (Fase 7)

- Consulta por CNPJ, razão social, porte, atividade
- Paginação e filtros avançados
- Cache de consultas frequentes

### Benchmark Automatizado (Fase 8)

- CI com benchmarks automáticos
- Comparação com versões anteriores
- Alertas de regressão de performance

### Plugin System (Fase 9)

- Novos destinos: ClickHouse, Elasticsearch, SQLite, Parquet, S3, BigQuery
- Interface de plugin padronizada
- Carregamento dinâmico de plugins
