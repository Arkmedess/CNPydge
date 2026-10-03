"""Orquestrador da esteira unitária de ingestão e processamento da RFB."""

from collections.abc import Callable
from pathlib import Path

import cnpydge
from cnpydge.crawler import WebDavCrawler
from cnpydge.downloader import download_file
from cnpydge.logging import get_logger
from cnpydge.pipeline.checkpoint_manifest_store import JsonCheckpointManifestStore
from cnpydge.pipeline.disk_cleanup_policy import create_disk_cleanup_policy
from cnpydge.pipeline.partition_models import (
    DiskRetentionProfile,
    PartitionProcessingRecord,
    RfbPartitionStage,
    RfbTablePartition,
)
from cnpydge.pipeline.pipeline_contracts import (
    DiskCleanupPolicy,
    PipelineCheckpointStore,
    RfbArchiveExtractor,
)
from cnpydge.pipeline.zip_archive_extractor import StandardZipArchiveExtractor

LOGGER = get_logger("cnpydge.pipeline.orchestrator")


class RfbPipelineOrchestrator:
    """Orquestra o ciclo de vida completo de descoberta, download, extração, conversão e descarte."""

    def __init__(
        self,
        output_directory: str | Path,
        working_directory: str | Path | None = None,
        retention_profile: DiskRetentionProfile = DiskRetentionProfile.BALANCED_ROLLING_EVICTION,
        checkpoint_store: PipelineCheckpointStore | None = None,
        archive_extractor: RfbArchiveExtractor | None = None,
        cleanup_policy: DiskCleanupPolicy | None = None,
        downloader_fn: Callable[[str, Path], Path] | None = None,
        converter_fn: Callable[[str | Path, str | Path, str], int] | None = None,
        crawler: WebDavCrawler | None = None,
    ) -> None:
        """Inicializa o orquestrador com injeção de dependências e configuração de diretórios."""
        self._output_dir: Path = Path(output_directory)
        self._work_dir: Path = Path(working_directory) if working_directory else self._output_dir / ".tmp_work"
        self._profile: DiskRetentionProfile = retention_profile

        self._checkpoint_store: PipelineCheckpointStore = (
            checkpoint_store or JsonCheckpointManifestStore(self._output_dir / "pipeline_checkpoint.json")
        )
        self._extractor: RfbArchiveExtractor = archive_extractor or StandardZipArchiveExtractor()
        self._cleanup_policy: DiskCleanupPolicy = cleanup_policy or create_disk_cleanup_policy(self._profile)
        self._downloader_fn = downloader_fn or download_file
        self._converter_fn = converter_fn or cnpydge.to_parquet
        self._crawler = crawler or WebDavCrawler()

        self._output_dir.mkdir(parents=True, exist_ok=True)
        self._work_dir.mkdir(parents=True, exist_ok=True)

    def process_partition(self, partition: RfbTablePartition) -> PartitionProcessingRecord:
        """Processa uma única partição através da esteira unitária com observabilidade pontual."""
        record = PartitionProcessingRecord(
            partition=partition,
            stage=RfbPartitionStage.PENDING_DOWNLOAD,
        )

        if self._checkpoint_store.is_partition_completed(partition.remote_filename):
            LOGGER.info("Partição já concluída no manifesto [SKIP]: '%s'", partition.remote_filename)
            record.stage = RfbPartitionStage.CONVERSION_COMPLETED
            return record

        downloaded_zip: Path = self._work_dir / partition.remote_filename
        extracted_csv: Path | None = None

        try:
            # 1. Download
            LOGGER.info("[%s] [1/3 BAIXANDO] -> '%s'", partition.remote_filename, downloaded_zip.name)
            self._downloader_fn(partition.download_url, downloaded_zip)
            record.stage = RfbPartitionStage.DOWNLOAD_COMPLETED

            # 2. Extração e descarte imediato do ZIP
            LOGGER.info("[%s] [2/3 EXTRAINDO] do arquivo compactado", partition.remote_filename)
            extracted_csv = self._extractor.extract_csv_from_zip(downloaded_zip, self._work_dir)
            record.stage = RfbPartitionStage.EXTRACTION_COMPLETED
            self._cleanup_policy.cleanup_downloaded_zip(downloaded_zip)

            # 3. Conversão Parquet e descarte do CSV
            parquet_name = f"{Path(partition.remote_filename).stem.lower()}.parquet"
            target_parquet = self._output_dir / parquet_name
            LOGGER.info("[%s] [3/3 CONVERTENDO] -> '%s'", partition.remote_filename, target_parquet.name)
            rows = self._converter_fn(extracted_csv, target_parquet, partition.table_canonical_name)

            record.stage = RfbPartitionStage.CONVERSION_COMPLETED
            record.rows_converted = rows
            record.parquet_output_path = target_parquet
            self._cleanup_policy.cleanup_extracted_csv(extracted_csv)

            self._checkpoint_store.record_partition_progress(record)
            LOGGER.info("[%s] [CONCLUÍDO] %d linhas gravadas com sucesso.", partition.remote_filename, rows)
            return record

        except Exception as err:
            LOGGER.error("[%s] [FALHA NA ESTEIRA] Motivo: %s", partition.remote_filename, err)
            record.stage = RfbPartitionStage.PROCESSING_FAILED
            record.error_message = str(err)
            self._checkpoint_store.record_partition_progress(record)

            # Limpeza defensiva de artefatos temporários em caso de exceção
            if downloaded_zip.exists():
                downloaded_zip.unlink(missing_ok=True)
            if extracted_csv and extracted_csv.exists():
                extracted_csv.unlink(missing_ok=True)
            raise

    def run_pipeline(
        self,
        base_url: str = "https://dadosabertos.rfb.gov.br/CNPJ/",
        table_filter: list[str] | None = None,
        max_partitions: int | None = None,
    ) -> list[PartitionProcessingRecord]:
        """Descobre os arquivos remotos via WebDAV e executa a esteira sequencial unitária."""
        LOGGER.info("Iniciando descoberta de arquivos no endpoint: '%s'", base_url)
        remote_files = self._crawler.catalog(base_url)

        normalized_filter = {t.lower() for t in table_filter} if table_filter else None
        partitions: list[RfbTablePartition] = []

        for rf in remote_files:
            if not rf.table_kind:
                continue
            if normalized_filter and rf.table_kind.lower() not in normalized_filter:
                continue

            partitions.append(
                RfbTablePartition(
                    remote_filename=rf.name,
                    table_canonical_name=rf.table_kind,
                    download_url=rf.url,
                    size_bytes=rf.size_bytes or 0,
                )
            )

        if max_partitions:
            partitions = partitions[:max_partitions]

        LOGGER.info("Total de partições selecionadas para ingestão: %d", len(partitions))
        results: list[PartitionProcessingRecord] = []

        for p in partitions:
            res = self.process_partition(p)
            results.append(res)

        return results
