"""Testes unitários para os modelos e contratos declarativos de domínio do pipeline."""

from cnpydge.pipeline.partition_models import (
    DiskRetentionProfile,
    PartitionProcessingRecord,
    RfbPartitionStage,
    RfbTablePartition,
)


def test_disk_retention_profile_values() -> None:
    """Valida os valores declarativos dos perfis de retenção em disco."""
    assert DiskRetentionProfile.BALANCED_ROLLING_EVICTION == "balanced_rolling_eviction"
    assert DiskRetentionProfile.PERFORMANCE_BATCH_RETAIN == "performance_batch_retain"
    assert DiskRetentionProfile.ARCHIVAL_PERMANENT_ZIP == "archival_permanent_zip"


def test_rfb_partition_stage_lifecycle() -> None:
    """Garante que os estágios do ciclo de vida da partição sejam unívocos."""
    assert RfbPartitionStage.PENDING_DOWNLOAD == "pending_download"
    assert RfbPartitionStage.DOWNLOAD_COMPLETED == "download_completed"
    assert RfbPartitionStage.EXTRACTION_COMPLETED == "extraction_completed"
    assert RfbPartitionStage.CONVERSION_COMPLETED == "conversion_completed"
    assert RfbPartitionStage.PROCESSING_FAILED == "processing_failed"


def test_rfb_table_partition_creation() -> None:
    """Valida a imutabilidade e atributos da entidade RfbTablePartition."""
    partition = RfbTablePartition(
        remote_filename="Empresas0.zip",
        table_canonical_name="empresas",
        download_url="https://dadosabertos.rfb.gov.br/CNPJ/Empresas0.zip",
        size_bytes=1024 * 1024 * 200,
    )
    assert partition.remote_filename == "Empresas0.zip"
    assert partition.table_canonical_name == "empresas"
    assert partition.size_bytes == 209715200


def test_partition_processing_record_default_state() -> None:
    """Valida a criação do registro de auditoria e status de processamento."""
    partition = RfbTablePartition(
        remote_filename="Socios0.zip",
        table_canonical_name="socios",
        download_url="https://dadosabertos.rfb.gov.br/CNPJ/Socios0.zip",
        size_bytes=5000,
    )
    record = PartitionProcessingRecord(
        partition=partition,
        stage=RfbPartitionStage.PENDING_DOWNLOAD,
    )
    assert record.stage == RfbPartitionStage.PENDING_DOWNLOAD
    assert record.parquet_output_path is None
    assert record.rows_converted == 0
    assert record.error_message is None
