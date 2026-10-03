"""Pacote de orquestração do CNPydge com motor nativo em Rust."""

import time
from pathlib import Path
from typing import Final, Literal

from cnpydge import _core
from cnpydge._core import version
from cnpydge.crawler import RemoteFileMetadata, WebDavCrawler
from cnpydge.downloader import download_file, download_files_concurrently
from cnpydge.logging import get_logger, setup_logging
from cnpydge.pipeline import (
    DiskRetentionProfile,
    PartitionProcessingRecord,
    RfbPartitionStage,
    RfbPipelineOrchestrator,
    RfbTablePartition,
)

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
    "DiskRetentionProfile",
    "PartitionProcessingRecord",
    "RemoteFileMetadata",
    "RfbPartitionStage",
    "RfbPipelineOrchestrator",
    "RfbTablePartition",
    "TableKindStr",
    "WebDavCrawler",
    "download_file",
    "download_files_concurrently",
    "get_logger",
    "run_pipeline",
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
    except Exception:
        LOGGER.exception(
            "Falha no motor nativo ao converter '%s' -> '%s' [%s]",
            src_path,
            dst_path,
            table_name,
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


def run_pipeline(
    output_dir: str | Path,
    tables: list[str] | None = None,
    profile: DiskRetentionProfile = DiskRetentionProfile.BALANCED_ROLLING_EVICTION,
    base_url: str = "https://dadosabertos.rfb.gov.br/CNPJ/",
    max_partitions: int | None = None,
) -> list[PartitionProcessingRecord]:
    """Executa a esteira unificada de ingestão, extração e conversão dos dados da RFB.

    Args:
        output_dir: Diretório de destino final dos arquivos Parquet gerados.
        tables: Lista de tabelas canônicas a filtrar (ex.: ['empresas', 'socios']).
            Se None, processa todas as tabelas encontradas.
        profile: Perfil de retenção de armazenamento em disco temporário.
            O padrão é BALANCED_ROLLING_EVICTION (baixa, extrai, converte e expurga).
        base_url: URL base do repositório de dados abertos da Receita Federal.
        max_partitions: Limite opcional de partições a processar (útil para testes).

    Returns:
        Lista com os registros de auditoria de cada partição processada.
    """
    orchestrator = RfbPipelineOrchestrator(
        output_directory=output_dir,
        retention_profile=profile,
    )
    return orchestrator.run_pipeline(
        base_url=base_url,
        table_filter=tables,
        max_partitions=max_partitions,
    )
