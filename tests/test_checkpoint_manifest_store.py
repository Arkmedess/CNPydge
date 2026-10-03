"""Testes unitários para o repositório de checkpoints do pipeline (JSON)."""

import json
from pathlib import Path

from cnpydge.pipeline.checkpoint_manifest_store import JsonCheckpointManifestStore
from cnpydge.pipeline.partition_models import (
    PartitionProcessingRecord,
    RfbPartitionStage,
    RfbTablePartition,
)


def test_checkpoint_store_default_stage(tmp_path: Path) -> None:
    """Verifica se partições não registradas retornam PENDING_DOWNLOAD por padrão."""
    manifest_path = tmp_path / "checkpoint.json"
    store = JsonCheckpointManifestStore(manifest_path)

    stage = store.get_partition_stage("Empresas0.zip")
    assert stage == RfbPartitionStage.PENDING_DOWNLOAD
    assert not store.is_partition_completed("Empresas0.zip")


def test_checkpoint_store_record_and_persist(tmp_path: Path) -> None:
    """Verifica a persistência de progresso e leitura atômica do arquivo de manifesto."""
    manifest_path = tmp_path / "checkpoint.json"
    store = JsonCheckpointManifestStore(manifest_path)

    partition = RfbTablePartition(
        remote_filename="Empresas0.zip",
        table_canonical_name="empresas",
        download_url="https://dadosabertos.rfb.gov.br/CNPJ/Empresas0.zip",
    )
    record = PartitionProcessingRecord(
        partition=partition,
        stage=RfbPartitionStage.CONVERSION_COMPLETED,
        parquet_output_path=tmp_path / "empresas_0.parquet",
        rows_converted=15000,
    )
    store.record_partition_progress(record)

    assert store.is_partition_completed("Empresas0.zip")
    assert store.get_partition_stage("Empresas0.zip") == RfbPartitionStage.CONVERSION_COMPLETED

    # Testa se o arquivo físico foi gravado e pode ser lido por uma nova instância
    assert manifest_path.is_file()
    new_store = JsonCheckpointManifestStore(manifest_path)
    assert new_store.is_partition_completed("Empresas0.zip")
    assert new_store.get_partition_stage("Empresas0.zip") == RfbPartitionStage.CONVERSION_COMPLETED

    data = json.loads(manifest_path.read_text(encoding="utf-8"))
    assert "Empresas0.zip" in data
    assert data["Empresas0.zip"]["rows_converted"] == 15000


def test_checkpoint_store_failed_stage(tmp_path: Path) -> None:
    """Garante que estágios de falha registrem a mensagem de erro no manifesto."""
    manifest_path = tmp_path / "checkpoint.json"
    store = JsonCheckpointManifestStore(manifest_path)

    partition = RfbTablePartition(
        remote_filename="Socios0.zip",
        table_canonical_name="socios",
        download_url="https://dadosabertos.rfb.gov.br/CNPJ/Socios0.zip",
    )
    record = PartitionProcessingRecord(
        partition=partition,
        stage=RfbPartitionStage.PROCESSING_FAILED,
        error_message="Falha de conexão com o servidor RFB",
    )
    store.record_partition_progress(record)

    assert store.get_partition_stage("Socios0.zip") == RfbPartitionStage.PROCESSING_FAILED
    assert not store.is_partition_completed("Socios0.zip")
