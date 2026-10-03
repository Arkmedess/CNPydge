"""Contratos e interfaces abstratas do pipeline de processamento da RFB."""

from pathlib import Path
from typing import Protocol

from cnpydge.pipeline.partition_models import (
    PartitionProcessingRecord,
    RfbPartitionStage,
)


class PipelineCheckpointStore(Protocol):
    """Contrato declarativo para persistência e recuperação do manifesto de estado."""

    def get_partition_stage(self, partition_filename: str) -> RfbPartitionStage:
        """Recupera o estágio operacional atual da partição."""
        ...

    def record_partition_progress(self, record: PartitionProcessingRecord) -> None:
        """Persiste o progresso e dados de auditoria da partição."""
        ...

    def is_partition_completed(self, partition_filename: str) -> bool:
        """Verifica se a partição já foi convertida para Parquet com sucesso."""
        ...


class RfbArchiveExtractor(Protocol):
    """Contrato declarativo para extração atômica de arquivos da Receita Federal."""

    def extract_csv_from_zip(
        self,
        zip_archive_path: Path,
        destination_directory: Path,
    ) -> Path:
        """Extrai o arquivo CSV contido no arquivo ZIP da RFB e retorna seu caminho."""
        ...


class DiskCleanupPolicy(Protocol):
    """Contrato declarativo que governa a política de expurgo de arquivos intermediários."""

    def cleanup_downloaded_zip(self, downloaded_zip_path: Path) -> None:
        """Aplica a regra de descarte sobre o arquivo compactado após a extração."""
        ...

    def cleanup_extracted_csv(self, extracted_csv_path: Path) -> None:
        """Aplica a regra de descarte sobre o CSV intermediário após a conversão Parquet."""
        ...
