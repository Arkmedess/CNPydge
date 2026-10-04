"""Políticas declarativas de retenção e expurgo de arquivos em disco temporário."""

from pathlib import Path

from cnpydge.logging import get_logger
from cnpydge.pipeline.partition_models import DiskRetentionProfile
from cnpydge.pipeline.pipeline_contracts import DiskCleanupPolicy

LOGGER = get_logger("cnpydge.pipeline.cleanup")


def _unlink_and_log(path: Path, log_template: str) -> None:
    """Remove o arquivo e registra a ocorrência no logger."""
    if path.exists():
        path.unlink(missing_ok=True)
        LOGGER.info(log_template, path.name)


class BalancedRollingEvictionPolicy:
    """Esteira unitária: remove o ZIP após extração e o CSV após conversão Parquet."""

    def cleanup_downloaded_zip(self, downloaded_zip_path: Path) -> None:
        """Exclui o arquivo compactado imediatamente para liberar espaço no disco."""
        _unlink_and_log(downloaded_zip_path, "Arquivo compactado descartado [Rolling Eviction]: '%s'")

    def cleanup_extracted_csv(self, extracted_csv_path: Path) -> None:
        """Exclui o CSV intermediário imediatamente após a conversão."""
        _unlink_and_log(extracted_csv_path, "CSV intermediário descartado [Rolling Eviction]: '%s'")


class ArchivalPermanentZipPolicy:
    """Política de auditoria: preserva os arquivos ZIP originais e descarta apenas o CSV."""

    def cleanup_downloaded_zip(self, downloaded_zip_path: Path) -> None:
        """Mantém o ZIP intacto para fins de histórico e auditoria legal."""
        LOGGER.info("Arquivo compactado preservado [Archival]: '%s'", downloaded_zip_path.name)

    def cleanup_extracted_csv(self, extracted_csv_path: Path) -> None:
        """Descarta o CSV temporário pois os dados já foram persistidos em Parquet."""
        _unlink_and_log(extracted_csv_path, "CSV intermediário descartado [Archival]: '%s'")


class PerformanceBatchRetainPolicy:
    """Política para servidores de alto throughput: preserva intermediários para lote."""

    def cleanup_downloaded_zip(self, downloaded_zip_path: Path) -> None:
        """Preserva o arquivo ZIP durante o processamento em lote."""
        LOGGER.debug("ZIP preservado para execução em lote: '%s'", downloaded_zip_path.name)

    def cleanup_extracted_csv(self, extracted_csv_path: Path) -> None:
        """Preserva o arquivo CSV para execução em lote."""
        LOGGER.debug("CSV preservado para execução em lote: '%s'", extracted_csv_path.name)


def create_disk_cleanup_policy(profile: DiskRetentionProfile) -> DiskCleanupPolicy:
    """Fábrica declarativa que instancia a política apropriada para o perfil selecionado."""
    match profile:
        case DiskRetentionProfile.BALANCED_ROLLING_EVICTION:
            return BalancedRollingEvictionPolicy()
        case DiskRetentionProfile.ARCHIVAL_PERMANENT_ZIP:
            return ArchivalPermanentZipPolicy()
        case DiskRetentionProfile.PERFORMANCE_BATCH_RETAIN:
            return PerformanceBatchRetainPolicy()
