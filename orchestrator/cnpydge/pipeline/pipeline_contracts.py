"""Contratos e interfaces abstratas do pipeline de processamento da RFB."""

from pathlib import Path
from typing import Protocol

from cnpydge.pipeline.partition_models import (
    PartitionProcessingRecord,
    RfbPartitionStage,
    RfbTablePartition,
)


class PartitionDiscoveryProvider(Protocol):
    """Contrato declarativo para descoberta e catalogação de partições a processar."""

    def discover_partitions(
        self,
        target_location: str,
        table_filter: list[str] | None = None,
        max_partitions: int | None = None,
    ) -> list[RfbTablePartition]:
        """Descobre recursos disponíveis e retorna a lista de partições normalizadas.

        Args:
            target_location: Localização ou endpoint alvo (ex.: URL WebDAV, caminho local, bucket).
            table_filter: Filtro opcional de identificadores canônicos de tabelas.
            max_partitions: Limite opcional de partições a serem retornadas.

        Returns:
            Lista de partições identificadas e mapeadas prontas para processamento.
        """
        ...


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
