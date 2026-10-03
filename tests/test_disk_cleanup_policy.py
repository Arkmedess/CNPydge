"""Testes unitários para as políticas de limpeza e retenção em disco."""

from pathlib import Path

from cnpydge.pipeline.disk_cleanup_policy import (
    ArchivalPermanentZipPolicy,
    BalancedRollingEvictionPolicy,
    PerformanceBatchRetainPolicy,
    create_disk_cleanup_policy,
)
from cnpydge.pipeline.partition_models import DiskRetentionProfile


def test_balanced_rolling_eviction_removes_both(tmp_path: Path) -> None:
    """Verifica se a esteira unitária remove o ZIP logo após a extração e o CSV após a conversão."""
    policy = BalancedRollingEvictionPolicy()

    zip_file = tmp_path / "Empresas0.zip"
    zip_file.write_text("dummy zip content")
    csv_file = tmp_path / "Empresas0.csv"
    csv_file.write_text("dummy csv content")

    policy.cleanup_downloaded_zip(zip_file)
    assert not zip_file.exists()

    policy.cleanup_extracted_csv(csv_file)
    assert not csv_file.exists()


def test_archival_permanent_zip_preserves_zip(tmp_path: Path) -> None:
    """Verifica se a política de arquivo mantém o ZIP original e deleta apenas o CSV."""
    policy = ArchivalPermanentZipPolicy()

    zip_file = tmp_path / "Socios0.zip"
    zip_file.write_text("dummy zip content")
    csv_file = tmp_path / "Socios0.csv"
    csv_file.write_text("dummy csv content")

    policy.cleanup_downloaded_zip(zip_file)
    assert zip_file.exists()

    policy.cleanup_extracted_csv(csv_file)
    assert not csv_file.exists()


def test_performance_batch_retain_preserves_both(tmp_path: Path) -> None:
    """Verifica se a política de performance preserva os arquivos para execução em lote."""
    policy = PerformanceBatchRetainPolicy()

    zip_file = tmp_path / "Cnaes.zip"
    zip_file.write_text("dummy zip content")
    csv_file = tmp_path / "Cnaes.csv"
    csv_file.write_text("dummy csv content")

    policy.cleanup_downloaded_zip(zip_file)
    assert zip_file.exists()

    policy.cleanup_extracted_csv(csv_file)
    assert csv_file.exists()


def test_factory_creates_correct_policy() -> None:
    """Valida a resolução da política a partir do enum declarativo."""
    assert isinstance(create_disk_cleanup_policy(DiskRetentionProfile.BALANCED_ROLLING_EVICTION), BalancedRollingEvictionPolicy)
    assert isinstance(create_disk_cleanup_policy(DiskRetentionProfile.ARCHIVAL_PERMANENT_ZIP), ArchivalPermanentZipPolicy)
    assert isinstance(create_disk_cleanup_policy(DiskRetentionProfile.PERFORMANCE_BATCH_RETAIN), PerformanceBatchRetainPolicy)
