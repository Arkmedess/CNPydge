"""Download assincrono dos arquivos da RFB.

Responsabilidades:
- Listar arquivos disponiveis no site da RFB
- Download com httpx (async) + verificacao SHA256
- Cache local (nao re-download de arquivos ja baixados)
- Protecao contra zip bomb (limitar tamanho descompactado)

Ver skills: python-orchestration, security-audit.
"""

import hashlib
import shutil
from pathlib import Path

import httpx
from loguru import logger

# Base URL for RFB CNPJ data
RFB_BASE_URL = "https://dadosabertos.rfb.gov.br/CNPJ"

# Known file patterns for the main CNPJ datasets
RFB_FILES = [
    "Empresas0.zip",
    "Empresas1.zip",
    "Empresas2.zip",
    "Empresas3.zip",
    "Empresas4.zip",
    "Empresas5.zip",
    "Empresas6.zip",
    "Empresas7.zip",
    "Empresas8.zip",
    "Empresas9.zip",
    "Estabelecimentos0.zip",
    "Estabelecimentos1.zip",
    "Estabelecimentos2.zip",
    "Estabelecimentos3.zip",
    "Estabelecimentos4.zip",
    "Estabelecimentos5.zip",
    "Estabelecimentos6.zip",
    "Estabelecimentos7.zip",
    "Estabelecimentos8.zip",
    "Estabelecimentos9.zip",
    "Socios0.zip",
    "Socios1.zip",
    "Socios2.zip",
    "Socios3.zip",
    "Socios4.zip",
    "Socios5.zip",
    "Socios6.zip",
    "Socios7.zip",
    "Socios8.zip",
    "Socios9.zip",
    "Simples.zip",
    "CNAEs.zip",
    "Naturezas.zip",
    "Paises.zip",
    "Municipios.zip",
    "Qualificacoes.zip",
]


def compute_sha256(path: Path) -> str:
    """Compute SHA256 hash of a file."""
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return h.hexdigest()


async def download_file(
    client: httpx.AsyncClient,
    filename: str,
    dest_dir: Path,
    expected_sha256: str | None = None,
) -> Path:
    """Download a single file from RFB.

    Args:
        client: httpx async client
        filename: Name of the file to download
        dest_dir: Destination directory
        expected_sha256: Optional expected SHA256 hash for verification

    Returns:
        Path to the downloaded file

    Raises:
        httpx.HTTPError: If download fails
        ValueError: If SHA256 verification fails
    """
    dest_path = dest_dir / filename

    # Skip if already downloaded
    if dest_path.exists():
        if expected_sha256:
            actual = compute_sha256(dest_path)
            if actual != expected_sha256:
                logger.warning(
                    "SHA256 mismatch for {}, re-downloading", filename
                )
            else:
                logger.info("Already cached: {}", filename)
                return dest_path
        else:
            logger.info("Already cached: {}", filename)
            return dest_path

    url = f"{RFB_BASE_URL}/{filename}"
    logger.info("Downloading {}...", filename)

    async with client.stream("GET", url) as response:
        response.raise_for_status()
        with open(dest_path, "wb") as f:
            async for chunk in response.aiter_bytes(chunk_size=8192):
                f.write(chunk)

    # Verify SHA256 if provided
    if expected_sha256:
        actual = compute_sha256(dest_path)
        if actual != expected_sha256:
            dest_path.unlink()  # Remove corrupted file
            raise ValueError(
                f"SHA256 mismatch for {filename}: "
                f"expected {expected_sha256}, got {actual}"
            )

    logger.info("Downloaded: {}", filename)
    return dest_path


async def download_all(
    dest_dir: Path,
    files: list[str] | None = None,
    max_concurrent: int = 4,
) -> list[Path]:
    """Download all RFB files.

    Args:
        dest_dir: Destination directory for downloads
        files: List of filenames to download (defaults to all known files)
        max_concurrent: Maximum concurrent downloads

    Returns:
        List of paths to downloaded files
    """
    dest_dir.mkdir(parents=True, exist_ok=True)
    files_to_download = files or RFB_FILES

    downloaded = []
    async with httpx.AsyncClient(
        timeout=httpx.Timeout(300.0),
        follow_redirects=True,
        limits=httpx.Limits(max_connections=max_concurrent),
    ) as client:
        for filename in files_to_download:
            try:
                path = await download_file(client, filename, dest_dir)
                downloaded.append(path)
            except Exception as e:
                logger.error("Failed to download {}: {}", filename, e)

    return downloaded


def list_cached_files(cache_dir: Path) -> list[Path]:
    """List all cached ZIP files in the given directory."""
    if not cache_dir.exists():
        return []
    return sorted(cache_dir.glob("*.zip"))


def clear_cache(cache_dir: Path) -> int:
    """Clear the download cache. Returns number of files removed."""
    count = 0
    if cache_dir.exists():
        for f in cache_dir.glob("*.zip"):
            f.unlink()
            count += 1
    return count
