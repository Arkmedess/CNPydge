"""Testes unitários para o provedor de descoberta WebDavPartitionDiscoveryProvider."""

from unittest.mock import MagicMock

from cnpydge.network import RemoteFileMetadata, WebDavCrawler
from cnpydge.pipeline.partition_models import RfbTableRegistry
from cnpydge.pipeline.webdav_partition_discovery import WebDavPartitionDiscoveryProvider


def test_webdav_discovery_provider_filters_and_normalizes() -> None:
    """Garante que o provedor converte RemoteFileMetadata em RfbTablePartition filtradas."""
    mock_crawler = MagicMock(spec=WebDavCrawler)
    mock_crawler.catalog.return_value = [
        RemoteFileMetadata(
            name="Empresas0.zip", url="https://mock/Empresas0.zip", size_bytes=100
        ),
        RemoteFileMetadata(
            name="Socios0.zip", url="https://mock/Socios0.zip", size_bytes=200
        ),
        RemoteFileMetadata(
            name="arquivo_estranho.txt",
            url="https://mock/arquivo_estranho.txt",
            size_bytes=50,
        ),
    ]

    provider = WebDavPartitionDiscoveryProvider(crawler=mock_crawler)
    partitions = provider.discover_partitions(
        target_location="https://mock/",
        table_filter=["empresas"],
    )

    assert len(partitions) == 1
    assert partitions[0].table_canonical_name == "empresas"
    assert partitions[0].remote_filename == "Empresas0.zip"
    assert partitions[0].download_url == "https://mock/Empresas0.zip"
    assert partitions[0].size_bytes == 100


def test_webdav_discovery_provider_max_partitions_limit() -> None:
    """Valida o truncamento do resultado ao aplicar max_partitions."""
    mock_crawler = MagicMock(spec=WebDavCrawler)
    mock_crawler.catalog.return_value = [
        RemoteFileMetadata(
            name="Empresas0.zip", url="https://mock/Empresas0.zip", size_bytes=100
        ),
        RemoteFileMetadata(
            name="Empresas1.zip", url="https://mock/Empresas1.zip", size_bytes=100
        ),
        RemoteFileMetadata(
            name="Empresas2.zip", url="https://mock/Empresas2.zip", size_bytes=100
        ),
    ]

    provider = WebDavPartitionDiscoveryProvider(crawler=mock_crawler)
    partitions = provider.discover_partitions(
        target_location="https://mock/",
        max_partitions=2,
    )

    assert len(partitions) == 2
    assert [p.remote_filename for p in partitions] == ["Empresas0.zip", "Empresas1.zip"]


def test_webdav_discovery_provider_custom_registry() -> None:
    """Garante suporte a catálogos customizados injetados (OCP)."""
    mock_crawler = MagicMock(spec=WebDavCrawler)
    mock_crawler.catalog.return_value = [
        RemoteFileMetadata(
            name="TabelaInovadora.zip",
            url="https://mock/TabelaInovadora.zip",
            size_bytes=300,
        ),
    ]

    custom_registry = RfbTableRegistry(prefix_map={"tabelainovadora": "inovadora"})
    provider = WebDavPartitionDiscoveryProvider(
        crawler=mock_crawler,
        table_registry=custom_registry,
    )
    partitions = provider.discover_partitions(target_location="https://mock/")

    assert len(partitions) == 1
    assert partitions[0].table_canonical_name == "inovadora"
