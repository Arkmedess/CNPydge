"""Módulo de download em streaming com suporte a retomada (Range) e concorrência."""

import time
from collections.abc import Callable
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Final

import httpx

from cnpydge.logging import get_logger

LOGGER = get_logger("cnpydge.downloader")

DEFAULT_CHUNK_SIZE_BYTES: Final[int] = 1024 * 1024  # 1 MB


def download_file(
    url: str,
    destination: str | Path,
    auth: tuple[str, str] | None = None,
    resume: bool = True,
    chunk_size: int = DEFAULT_CHUNK_SIZE_BYTES,
    client: httpx.Client | None = None,
    on_progress: Callable[[int, int | None], None] | None = None,
) -> Path:
    """Realiza o download de um arquivo remoto via HTTP streaming com atomicidade e resume.

    Args:
        url: URL direta do arquivo remoto no WebDAV/HTTP.
        destination: Caminho local de destino do arquivo final.
        auth: Tupla opcional (usuário, senha) para autenticação Basic.
        resume: Se True, tenta retomar download incompleto via cabeçalho HTTP Range.
        chunk_size: Tamanho de cada bloco lido e gravado em disco (padrão: 1 MB).
        client: Cliente HTTP opcional (se None, cria e gerencia uma conexão).
        on_progress: Callback opcional recebendo (bytes_baixados, total_bytes).

    Returns:
        Path do arquivo salvo localmente após validação.

    Raises:
        httpx.HTTPStatusError: Se a resposta HTTP contiver código de erro 4xx ou 5xx.
        httpx.RequestError: Se ocorrer falha de conexão, timeout ou erro de transporte.
        OSError: Se houver falha de criação de diretórios ou gravação no disco.
    """
    dest_path: Path = Path(destination)
    dest_path.parent.mkdir(parents=True, exist_ok=True)
    part_path: Path = dest_path.parent / f"{dest_path.name}.part"

    managed_client: bool = client is None
    http_client: httpx.Client = client or httpx.Client(timeout=120.0)
    auth_kwarg = auth if auth is not None else httpx.USE_CLIENT_DEFAULT

    try:
        # 1. Verifica se o arquivo final já existe e confere tamanho
        if dest_path.is_file():
            try:
                head_resp = http_client.head(url, auth=auth_kwarg)
                if head_resp.status_code == 200:
                    remote_len_header = head_resp.headers.get("Content-Length")
                    if (
                        remote_len_header
                        and int(remote_len_header) == dest_path.stat().st_size
                    ):
                        LOGGER.info(
                            "Arquivo '%s' já existe e está íntegro. Download ignorado.",
                            dest_path,
                        )
                        return dest_path
            except httpx.RequestError:
                pass

        # 2. Configura retomada parcial (HTTP Range)
        headers: dict[str, str] = {}
        downloaded_bytes = 0
        file_mode = "wb"

        if resume and part_path.is_file():
            downloaded_bytes = part_path.stat().st_size
            if downloaded_bytes > 0:
                headers["Range"] = f"bytes={downloaded_bytes}-"
                file_mode = "ab"
                LOGGER.info(
                    "Retomando download de '%s' a partir de %d bytes via HTTP Range.",
                    part_path.name,
                    downloaded_bytes,
                )

        start_time: float = time.perf_counter()

        with http_client.stream("GET", url, auth=auth_kwarg, headers=headers) as response:
            response.raise_for_status()

            is_partial: bool = response.status_code == 206
            if not is_partial and file_mode == "ab":
                # Servidor não suportou Range; reinicia download do zero
                file_mode = "wb"
                downloaded_bytes = 0

            content_len_header = response.headers.get("Content-Length")
            total_expected_bytes: int | None = None
            if content_len_header:
                total_expected_bytes = int(content_len_header) + (
                    downloaded_bytes if is_partial else 0
                )

            with part_path.open(file_mode) as file_handle:
                for chunk in response.iter_bytes(chunk_size=chunk_size):
                    if not chunk:
                        continue
                    file_handle.write(chunk)
                    downloaded_bytes += len(chunk)
                    if on_progress:
                        on_progress(downloaded_bytes, total_expected_bytes)


        elapsed: float = max(time.perf_counter() - start_time, 1e-9)
        tput_mb: float = (downloaded_bytes / (1024 * 1024)) / elapsed

        # 3. Renomeação atômica do arquivo temporário para o destino final
        part_path.replace(dest_path)

        LOGGER.info(
            "Download concluído: '%s' (%d bytes baixados em %.2fs, vazão: %.2f MB/s).",
            dest_path.name,
            downloaded_bytes,
            elapsed,
            tput_mb,
        )
        return dest_path

    finally:
        if managed_client:
            http_client.close()


def download_files_concurrently(
    items: list[tuple[str, Path]],
    auth: tuple[str, str] | None = None,
    max_workers: int = 4,
    chunk_size: int = DEFAULT_CHUNK_SIZE_BYTES,
    client: httpx.Client | None = None,
) -> list[Path]:
    """Baixa múltiplos arquivos em paralelo utilizando threads (sem bloqueio de GIL).

    Args:
        items: Lista de pares (url_remota, destino_local).
        auth: Credenciais de autenticação opcionais.
        max_workers: Número máximo de downloads simultâneos (padrão: 4).
        chunk_size: Tamanho de cada bloco lido e gravado em disco.
        client: Cliente HTTP compartilhado opcional.

    Returns:
        Lista de caminhos dos arquivos baixados com sucesso.

    Raises:
        httpx.HTTPStatusError: Se qualquer um dos downloads receber status HTTP de erro.
        httpx.RequestError: Se houver falha irrecuperável de rede durante a execução paralela.
        Exception: Repassa qualquer exceção não tratada levantada nos workers.
    """
    LOGGER.info(
        "Iniciando download concorrente de %d arquivos com %d workers em paralelo.",
        len(items),
        max_workers,
    )

    results: list[Path] = []
    with ThreadPoolExecutor(max_workers=max_workers) as executor:
        future_to_item = {
            executor.submit(
                download_file,
                url=url,
                destination=dst,
                auth=auth,
                chunk_size=chunk_size,
                client=client,
            ): dst
            for url, dst in items
        }

        for future in as_completed(future_to_item):
            dst: Path = future_to_item[future]
            try:
                downloaded_path: Path = future.result()
                results.append(downloaded_path)
            except Exception as err:
                LOGGER.error("Falha ao baixar '%s': %s", dst, err)
                raise

    return results
