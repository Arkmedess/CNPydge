"""Testes de conformidade e integridade das fronteiras arquiteturais do orquestrador.

Valida a Issue #8:
- Submódulo canônico cnpydge.network (WebDavCrawler, download_file).
- Submódulo canônico cnpydge.api (to_parquet, run_pipeline).
- Submódulo cnpydge.catalog inicializado.
- Fachada pública cnpydge na raiz com 100% de retrocompatibilidade.
- Aliases legados cnpydge.crawler e cnpydge.downloader preservados.
"""

import sys
import types
from typing import Any
from unittest.mock import MagicMock


def _ensure_mocked_core() -> None:
    """Garante que cnpydge._core esteja disponível no ambiente de teste unitário puro."""
    if "cnpydge._core" not in sys.modules:
        core_mock: Any = types.ModuleType("cnpydge._core")
        core_mock.version = lambda: "0.2.0"
        core_mock.to_parquet = MagicMock(return_value=100)
        sys.modules["cnpydge._core"] = core_mock


def test_public_facade_exports_canonical_contracts() -> None:
    """Garante que cnpydge.__init__ mantenha todos os contratos públicos previstos."""
    _ensure_mocked_core()
    import cnpydge

    expected_symbols = [
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

    for symbol in expected_symbols:
        assert hasattr(cnpydge, symbol), (
            f"Fachada cnpydge perdeu o símbolo público: '{symbol}'"
        )
        assert symbol in cnpydge.__all__, (
            f"Símbolo '{symbol}' não está declarado em cnpydge.__all__"
        )


def test_network_submodule_exports_and_isolation() -> None:
    """Garante que cnpydge.network encapsule crawler e downloader."""
    _ensure_mocked_core()
    import cnpydge.network as net

    assert hasattr(net, "WebDavCrawler")
    assert hasattr(net, "RemoteFileMetadata")
    assert hasattr(net, "download_file")
    assert hasattr(net, "download_files_concurrently")
    assert "WebDavCrawler" in net.__all__
    assert "download_file" in net.__all__


def test_api_submodule_exports() -> None:
    """Garante que cnpydge.api concentre os pontos de entrada do pipeline e conversão."""
    _ensure_mocked_core()
    from cnpydge import api

    assert hasattr(api, "to_parquet")
    assert hasattr(api, "run_pipeline")
    assert hasattr(api, "VALID_KINDS")
    assert "to_parquet" in api.__all__
    assert "run_pipeline" in api.__all__


def test_catalog_submodule_initialization() -> None:
    """Garante que o submódulo cnpydge.catalog exista como fronteira declarada."""
    _ensure_mocked_core()
    from cnpydge import catalog

    assert catalog is not None


def test_legacy_modules_backward_compatibility() -> None:
    """Garante que importações legadas cnpydge.crawler e cnpydge.downloader continuem funcionando."""
    _ensure_mocked_core()
    from cnpydge.crawler import RemoteFileMetadata as LegacyRemoteFileMetadata
    from cnpydge.crawler import WebDavCrawler as LegacyWebDavCrawler
    from cnpydge.downloader import download_file as legacy_download_file
    from cnpydge.downloader import (
        download_files_concurrently as legacy_download_files_concurrently,
    )
    from cnpydge.network import (
        RemoteFileMetadata,
        WebDavCrawler,
        download_file,
        download_files_concurrently,
    )

    assert LegacyWebDavCrawler is WebDavCrawler
    assert LegacyRemoteFileMetadata is RemoteFileMetadata
    assert legacy_download_file is download_file
    assert legacy_download_files_concurrently is download_files_concurrently