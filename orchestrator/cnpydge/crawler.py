"""Módulo legado mantido para compatibilidade retroativa.

Prefira importar diretamente de `cnpydge.network` ou `cnpydge.network.crawler`.
"""

from cnpydge.network.crawler import (
    DAV_NS,
    TABLE_PREFIX_MAP,
    RemoteFileMetadata,
    WebDavCrawler,
)

__all__: list[str] = [
    "DAV_NS",
    "TABLE_PREFIX_MAP",
    "RemoteFileMetadata",
    "WebDavCrawler",
]
