# Pacote de Orquestração CNPydge (`orchestrator`)

Camada de automação e interface pública de alto nível em Python (>= 3.13) para ingestão, download concorrente e conversão da base pública de CNPJs da Receita Federal.

## 1. Propósito
O pacote `cnpydge` orquestra a esteira completa de ingestão de dados: cataloga recursivamente os arquivos disponibilizados pela Receita Federal via WebDAV, gerencia downloads concorrentes com retomada e invoca o motor nativo em Rust com métricas de desempenho e diagnóstico estruturado.

## 2. Contrato Público & Assinaturas

### A. Conversão de Dados (`cnpydge.to_parquet`)
```python
def to_parquet(
    src: str | Path,
    dst: str | Path,
    kind: TableKindStr | str | None = None,
) -> int: ...
```
- Valida a existência do arquivo fonte e se o identificador da tabela pertence às 10 entidades suportadas.
- Cria os diretórios de destino automaticamente (`mkdir(parents=True)`).
- Mede o tempo de processamento e emite logs estruturados com throughput em `linhas/s` e `MB/s`.

### B. Descoberta WebDAV (`cnpydge.crawler.WebDavCrawler`)
```python
class WebDavCrawler:
    def __init__(self, base_url: str | None = None, token: str | None = None, client: httpx.Client | None = None): ...
    def catalog(self, base_url: str, pattern: str = "*.zip") -> list[RemoteFileMetadata]: ...
```
- Realiza requisições `PROPFIND` com parsing tolerante a múltiplos namespaces XML.
- Retorna instâncias imutáveis de `RemoteFileMetadata` contendo nome, URL direta, tamanho em bytes e data da última modificação.

### C. Download em Streaming Concorrente (`cnpydge.downloader`)
- `download_file(...) -> Path`: Executa o download atômico em blocos de 1 MB, suportando retomada via cabeçalho HTTP `Range` caso exista um arquivo temporário `.part`.
- `download_files_concurrently(...) -> list[Path]`: Coordena o download em paralelo de múltiplos arquivos utilizando `ThreadPoolExecutor`.

### D. Logging Estruturado (`cnpydge.logging`)
- `setup_logging(level, log_to_file, log_dir, colored)`: Configura formatação consistente com timestamp RFC 3339 e níveis coloridos.

## 3. Invariantes Operacionais
1. **Atomicidade de Downloads:** O arquivo só é promovido de `.part` para a extensão definitiva após validação do `Content-Length`.
2. **Isolamento de Memória:** O streaming HTTP não carrega arquivos completos na memória principal do Python.

## 4. Dependências
- **`httpx >= 0.27.0`:** Cliente HTTP robusto para streaming e requisições WebDAV com suporte a pooling de conexões e timeouts configuráveis.

## 5. Testes e Cobertura
- **Execução dos testes:** `uv run pytest --cov=cnpydge`
- **Linters:** `uv run ruff check .`