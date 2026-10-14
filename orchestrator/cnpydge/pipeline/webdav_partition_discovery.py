"""Provedor de descoberta e catalogação de partições baseado em WebDAV."""

from cnpydge.crawler import WebDavCrawler
from cnpydge.logging import get_logger
from cnpydge.pipeline.partition_models import RfbTablePartition, RfbTableRegistry
from cnpydge.pipeline.pipeline_contracts import PartitionDiscoveryProvider

LOGGER = get_logger("cnpydge.pipeline.webdav_discovery")


class WebDavPartitionDiscoveryProvider(PartitionDiscoveryProvider):
    """Adaptador de descoberta de partições da Receita Federal via WebDAV."""

    def __init__(
        self,
        crawler: WebDavCrawler | None = None,
        table_registry: RfbTableRegistry | None = None,
    ) -> None:
        """Inicializa o provedor com injeção opcional de crawler e catálogo de tabelas.

        Args:
            crawler: Cliente WebDavCrawler para requisições remotas (ou None para padrão).
            table_registry: Catálogo para inferência canônica de tabelas (ou None para padrão).
        """
        self._crawler: WebDavCrawler = crawler or WebDavCrawler()
        self._registry: RfbTableRegistry = table_registry or RfbTableRegistry()

    def discover_partitions(
        self,
        target_location: str,
        table_filter: list[str] | None = None,
        max_partitions: int | None = None,
    ) -> list[RfbTablePartition]:
        """Descobre recursos via WebDAV e converte em partições normalizadas do pipeline.

        Args:
            target_location: URL base do endpoint WebDAV ou repositório remoto.
            table_filter: Lista opcional de tabelas desejadas em formato textual (ex.: ['empresas']).
            max_partitions: Limite máximo opcional de partições a retornar.

        Returns:
            Lista de partições da RFB validadas e prontas para ingestão.
        """
        LOGGER.info("Consultando arquivos no repositório WebDAV: '%s'", target_location)
        remote_files = self._crawler.catalog(base_url=target_location)

        normalized_filter: set[str] | None = (
            {t.lower() for t in table_filter} if table_filter else None
        )

        partitions: list[RfbTablePartition] = []
        for rf in remote_files:
            table_kind = self._registry.infer_table_kind(rf.name)
            if not table_kind:
                continue

            if normalized_filter and table_kind.lower() not in normalized_filter:
                continue

            partitions.append(
                RfbTablePartition(
                    remote_filename=rf.name,
                    table_canonical_name=table_kind,
                    download_url=rf.url,
                    size_bytes=rf.size_bytes,
                )
            )

        if max_partitions is not None and max_partitions > 0:
            partitions = partitions[:max_partitions]

        LOGGER.info(
            "Descoberta finalizada: %d partições selecionadas de um total de %d arquivos remotos.",
            len(partitions),
            len(remote_files),
        )
        return partitions
