"""Módulo legado mantido para compatibilidade retroativa.

Prefira importar diretamente de `cnpydge.network` ou `cnpydge.network.downloader`.
"""

from cnpydge.network.downloader import (
    DEFAULT_CHUNK_SIZE_BYTES,
    download_file,
    download_files_concurrently,
)

__all__: list[str] = [
    "DEFAULT_CHUNK_SIZE_BYTES",
    "download_file",
    "download_files_concurrently",
]
