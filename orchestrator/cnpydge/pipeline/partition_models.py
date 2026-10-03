"""Modelos e enums de domínio declarativos para o pipeline de dados da RFB."""

from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path


class DiskRetentionProfile(StrEnum):
    """Perfil declarativo de retenção de armazenamento em disco temporário."""

    BALANCED_ROLLING_EVICTION = "balanced_rolling_eviction"
    PERFORMANCE_BATCH_RETAIN = "performance_batch_retain"
    ARCHIVAL_PERMANENT_ZIP = "archival_permanent_zip"


class RfbPartitionStage(StrEnum):
    """Estágio do ciclo de vida operacional de uma partição da Receita Federal."""

    PENDING_DOWNLOAD = "pending_download"
    DOWNLOAD_COMPLETED = "download_completed"
    EXTRACTION_COMPLETED = "extraction_completed"
    CONVERSION_COMPLETED = "conversion_completed"
    PROCESSING_FAILED = "processing_failed"


@dataclass(frozen=True, slots=True)
class RfbTablePartition:
    """Metadados imutáveis de uma partição de tabela remota da Receita Federal."""

    remote_filename: str
    table_canonical_name: str
    download_url: str
    size_bytes: int = 0


@dataclass(slots=True)
class PartitionProcessingRecord:
    """Registro mutável de estado e auditoria de uma partição no pipeline."""

    partition: RfbTablePartition
    stage: RfbPartitionStage
    parquet_output_path: Path | None = None
    rows_converted: int = 0
    error_message: str | None = None
