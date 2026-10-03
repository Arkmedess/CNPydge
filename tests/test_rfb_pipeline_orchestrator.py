"""Testes de integração e orquestração do pipeline completo da RFB."""

import zipfile
from pathlib import Path
from unittest.mock import MagicMock, patch

import cnpydge
import pytest
from cnpydge.crawler import RemoteFileMetadata
from cnpydge.pipeline.checkpoint_manifest_store import JsonCheckpointManifestStore
from cnpydge.pipeline.disk_cleanup_policy import BalancedRollingEvictionPolicy
from cnpydge.pipeline.partition_models import (
    DiskRetentionProfile,
    PartitionProcessingRecord,
    RfbPartitionStage,
    RfbTablePartition,
)
from cnpydge.pipeline.rfb_pipeline_orchestrator import RfbPipelineOrchestrator
from cnpydge.pipeline.zip_archive_extractor import StandardZipArchiveExtractor


def test_orchestrator_process_single_partition_rolling_eviction(tmp_path: Path) -> None:
    """Verifica a execução ponta a ponta de uma partição com esteira unitária e descarte imediato."""
    work_dir = tmp_path / "work"
    output_dir = tmp_path / "parquet_output"
    manifest_path = tmp_path / "checkpoint.json"

    # Prepara um arquivo ZIP simulando o download da RFB
    work_dir.mkdir(parents=True)
    zip_path = work_dir / "Empresas0.zip"
    with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        zf.writestr("Empresas0.csv", '"12345678";"EMPRESA ALFA LTDA";"2062"\n')

    # Mock do downloader que apenas garante o arquivo local
    mock_downloader = MagicMock(return_value=zip_path)

    # Mock do conversor nativo que simula a escrita do Parquet
    def mock_to_parquet(src: str | Path, dst: str | Path, kind: str) -> int:
        Path(dst).write_bytes(b"PARQUET_MAGIC_BYTES")
        return 100

    orchestrator = RfbPipelineOrchestrator(
        output_directory=output_dir,
        working_directory=work_dir,
        retention_profile=DiskRetentionProfile.BALANCED_ROLLING_EVICTION,
        checkpoint_store=JsonCheckpointManifestStore(manifest_path),
        archive_extractor=StandardZipArchiveExtractor(),
        cleanup_policy=BalancedRollingEvictionPolicy(),
        downloader_fn=mock_downloader,
        converter_fn=mock_to_parquet,
    )

    partition = RfbTablePartition(
        remote_filename="Empresas0.zip",
        table_canonical_name="empresas",
        download_url="https://dadosabertos.rfb.gov.br/CNPJ/Empresas0.zip",
        size_bytes=1000,
    )

    record = orchestrator.process_partition(partition)

    assert record.stage == RfbPartitionStage.CONVERSION_COMPLETED
    assert record.rows_converted == 100
    assert record.parquet_output_path is not None
    assert record.parquet_output_path.is_file()

    # Na esteira unitária (rolling eviction), nem o ZIP nem o CSV devem sobrar no diretório de trabalho
    assert not zip_path.exists()
    assert not (work_dir / "Empresas0.csv").exists()


def test_orchestrator_skips_already_completed_partition(tmp_path: Path) -> None:
    """Garante que partições já registradas como concluídas no manifesto sejam ignoradas."""
    manifest_path = tmp_path / "checkpoint.json"
    store = JsonCheckpointManifestStore(manifest_path)

    partition = RfbTablePartition(
        remote_filename="Empresas0.zip",
        table_canonical_name="empresas",
        download_url="https://dadosabertos.rfb.gov.br/CNPJ/Empresas0.zip",
    )

    # Registra no manifesto como concluído
    store.record_partition_progress(
        PartitionProcessingRecord(
            partition=partition,
            stage=RfbPartitionStage.CONVERSION_COMPLETED,
            parquet_output_path=tmp_path / "empresas_0.parquet",
            rows_converted=50,
        )
    )

    mock_downloader = MagicMock()
    mock_converter = MagicMock()

    orchestrator = RfbPipelineOrchestrator(
        output_directory=tmp_path / "output",
        working_directory=tmp_path / "work",
        checkpoint_store=store,
        downloader_fn=mock_downloader,
        converter_fn=mock_converter,
    )

    record = orchestrator.process_partition(partition)

    assert record.stage == RfbPartitionStage.CONVERSION_COMPLETED
    mock_downloader.assert_not_called()
    mock_converter.assert_not_called()


def test_orchestrator_handles_exception_and_cleans_up(tmp_path: Path) -> None:
    """Verifica se uma falha de conversão registra PROCESSING_FAILED e limpa resíduos."""
    work_dir = tmp_path / "work"
    work_dir.mkdir(parents=True)
    zip_path = work_dir / "Socios0.zip"
    zip_path.write_text("corrupted zip")

    mock_downloader = MagicMock(return_value=zip_path)
    mock_extractor = MagicMock()
    mock_extractor.extract_csv_from_zip.side_effect = RuntimeError("Falha de descompressão")
    manifest_path = tmp_path / "checkpoint.json"
    store = JsonCheckpointManifestStore(manifest_path)

    orchestrator = RfbPipelineOrchestrator(
        output_directory=tmp_path / "output",
        working_directory=work_dir,
        checkpoint_store=store,
        archive_extractor=mock_extractor,
        downloader_fn=mock_downloader,
    )

    partition = RfbTablePartition(
        remote_filename="Socios0.zip",
        table_canonical_name="socios",
        download_url="https://dadosabertos.rfb.gov.br/CNPJ/Socios0.zip",
    )

    with pytest.raises(RuntimeError, match="Falha de descompressão"):
        orchestrator.process_partition(partition)

    assert store.get_partition_stage("Socios0.zip") == RfbPartitionStage.PROCESSING_FAILED
    assert not zip_path.exists()


def test_orchestrator_run_pipeline_with_filter_and_limit(tmp_path: Path) -> None:
    """Verifica o método de conveniência run_pipeline com filtragem de tabelas e limite."""
    mock_crawler = MagicMock()
    mock_crawler.catalog.return_value = [
        RemoteFileMetadata(name="Empresas0.zip", url="http://test/Empresas0.zip", size_bytes=100),
        RemoteFileMetadata(name="Empresas1.zip", url="http://test/Empresas1.zip", size_bytes=100),
        RemoteFileMetadata(name="Socios0.zip", url="http://test/Socios0.zip", size_bytes=100),
        RemoteFileMetadata(name="ignorado.txt", url="http://test/ignorado.txt", size_bytes=10),
    ]

    orchestrator = RfbPipelineOrchestrator(
        output_directory=tmp_path / "output",
        crawler=mock_crawler,
    )
    with patch.object(orchestrator, "process_partition") as mock_process:
        records = orchestrator.run_pipeline(
            table_filter=["empresas"],
            max_partitions=1,
        )

        assert len(records) == 1
        mock_process.assert_called_once()


def test_public_run_pipeline_wrapper(tmp_path: Path) -> None:
    """Verifica a invocação da função pública cnpydge.run_pipeline."""
    with patch("cnpydge.pipeline.rfb_pipeline_orchestrator.RfbPipelineOrchestrator.run_pipeline") as mock_run:
        mock_run.return_value = []
        res = cnpydge.run_pipeline(
            output_dir=tmp_path / "out",
            tables=["empresas"],
            max_partitions=2,
        )
        assert res == []
        mock_run.assert_called_once_with(
            base_url="https://dadosabertos.rfb.gov.br/CNPJ/",
            table_filter=["empresas"],
            max_partitions=2,
        )
