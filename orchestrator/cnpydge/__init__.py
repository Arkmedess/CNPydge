"""Pacote de orquestração do CNPydge com motor nativo em Rust."""

import time
from pathlib import Path
from typing import Final, Literal

from cnpydge import _core
from cnpydge._core import version
from cnpydge.crawler import RemoteFileMetadata, WebDavCrawler
from cnpydge.downloader import download_file, download_files_concurrently
from cnpydge.logging import get_logger, setup_logging

type TableKindStr = Literal[
    "empresas",
    "socios",
    "estabelecimentos",
    "simples",
    "cnaes",
    "motivos",
    "municipios",
    "naturezas",
    "paises",
    "qualificacoes",
]

__all__: list[str] = [
    "RemoteFileMetadata",
    "TableKindStr",
    "WebDavCrawler",
    "download_file",
    "download_files_concurrently",
    "get_logger",
    "setup_logging",
    "to_parquet",
    "version",
]

LOGGER = get_logger("cnpydge.converter")

VALID_KINDS: Final[set[str]] = {
    "empresas",
    "empresa",
    "socios",
    "socio",
    "estabelecimentos",
    "estabelecimento",
    "simples",
    "simples_nacional",
    "mei",
    "cnaes",
    "cnae",
    "motivos",
    "motivo",
    "municipios",
    "municipio",
    "naturezas",
    "natureza",
    "paises",
    "pais",
    "qualificacoes",
    "qualificacao",
}


def to_parquet(
    src: str | Path,
    dst: str | Path,
    kind: TableKindStr | str | None = None,
) -> int:
    """Converte um arquivo CSV do CNPJ para Parquet com métricas e diagnóstico.

    Valida a existência do arquivo de entrada e o tipo da tabela solicitada,
    cria a estrutura de diretórios do destino se necessário e aciona o motor
    nativo em Rust com liberação de GIL. Ao final, afere o tempo decorrido,
    o throughput em linhas por segundo e a vazão em megabytes por segundo.

    Args:
        src: Caminho do arquivo CSV de entrada no sistema de arquivos.
        dst: Caminho de destino do arquivo Parquet colunar a ser gerado.
        kind: Identificador da tabela do CNPJ (ex.: 'empresas', 'socios',
            'estabelecimentos', 'simples', 'cnaes', 'municipios', etc.).
            Se omitido, assume 'empresas'.

    Returns:
        Total de linhas/registros processados e gravados no arquivo Parquet.

    Raises:
        FileNotFoundError: Se o arquivo de origem especificado em `src` não existir.
        ValueError: Se a tabela fornecida em `kind` não for suportada pelo conversor.
        RuntimeError: Se ocorrer uma falha durante o parsing ou escrita no motor nativo.
    """
    src_path = Path(src)
    dst_path = Path(dst)
    table_name: str = str(kind or "empresas").lower()

    if not src_path.is_file():
        LOGGER.error(
            "Arquivo de origem não encontrado: '%s'. Impossível converter '%s'.",
            src_path,
            table_name,
        )
        raise FileNotFoundError(f"Arquivo de origem '{src_path}' não encontrado.")

    if table_name not in VALID_KINDS:
        LOGGER.error("Tabela '%s' não suportada pelo conversor.", table_name)
        raise ValueError(
            f"Tabela '{table_name}' não suportada. Opções válidas: {sorted(VALID_KINDS)}"
        )

    dst_path.parent.mkdir(parents=True, exist_ok=True)
    src_bytes: int = src_path.stat().st_size
    LOGGER.info(
        "Iniciando conversão [%s]: '%s' (%d bytes) -> '%s'",
        table_name,
        src_path,
        src_bytes,
        dst_path,
    )

    start: float = time.perf_counter()
    try:
        total_rows = _core.to_parquet(str(src_path), str(dst_path), kind=table_name)
    except Exception as err:
        LOGGER.error(
            "Falha no motor nativo ao converter '%s' -> '%s' [%s]: %s",
            src_path,
            dst_path,
            table_name,
            err,
        )
        raise

    elapsed: float = max(time.perf_counter() - start, 1e-9)
    dst_bytes: int = dst_path.stat().st_size if dst_path.exists() else 0
    tput_rows: float = total_rows / elapsed
    tput_mb: float = (src_bytes / (1024 * 1024)) / elapsed

    LOGGER.info(
        "Conversão concluída [%s]: %d linhas processadas em %.3fs (%.1f linhas/s, %.2f MB/s). Saída: %d bytes.",
        table_name,
        total_rows,
        elapsed,
        tput_rows,
        tput_mb,
        dst_bytes,
    )
    return total_rows
