"""Submódulo de rede, descoberta WebDAV e download concorrente do CNPydge."""

from cnpydge.network.crawler import RemoteFileMetadata, WebDavCrawler
from cnpydge.network.downloader import download_file, download_files_concurrently

__all__: list[str] = [
    "RemoteFileMetadata",
    "WebDavCrawler",
    "download_file",
    "download_files_concurrently",
]

