"""Pacote de orquestração do CNPydge com motor nativo em Rust."""

from cnpydge._core import version
from cnpydge.api import TableKindStr, run_pipeline, to_parquet
from cnpydge.logging import get_logger, setup_logging
from cnpydge.network import (
    RemoteFileMetadata,
    WebDavCrawler,
    download_file,
    download_files_concurrently,
)
from cnpydge.pipeline import (
    DiskRetentionProfile,
    PartitionProcessingRecord,
    RfbPartitionStage,
    RfbPipelineOrchestrator,
    RfbTablePartition,
)

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
