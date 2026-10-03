"""Suíte de testes rigorosa para o downloader com streaming, resume e tolerância a falhas."""

from pathlib import Path

import httpx
import pytest
from cnpydge.downloader import (
    download_file,
    download_files_concurrently,
)


def test_download_file_complete_streaming(tmp_path: Path) -> None:
    """Valida download completo com escrita atômica via arquivo .part e callbacks de progresso."""
    content: bytes = b"conteudo de teste compactado em zip 12345"

    def handler(request: httpx.Request) -> httpx.Response:
        assert request.method == "GET"
        return httpx.Response(
            200,
            content=content,
            headers={"Content-Length": str(len(content))},
        )

    transport = httpx.MockTransport(handler)
    client = httpx.Client(transport=transport)
    dest = tmp_path / "empresas.zip"
    progress_records: list[tuple[int, int | None]] = []

    def on_progress(downloaded: int, total: int | None) -> None:
        progress_records.append((downloaded, total))

    result = download_file(
        url="https://example.com/empresas.zip",
        destination=dest,
        client=client,
        on_progress=on_progress,
        chunk_size=10,
    )

    assert result == dest
    assert dest.is_file()
    assert dest.read_bytes() == content
    assert not dest.with_suffix(".zip.part").exists()
    assert len(progress_records) > 0
    assert progress_records[-1] == (len(content), len(content))


def test_download_file_resume_with_http_range(tmp_path: Path) -> None:
    """Valida retomada de download incompleto com cabeçalho Range e resposta 206."""
    full_content: bytes = b"0123456789ABCDEF"
    part_initial_content: bytes = b"01234"  # Primeiros 5 bytes já existentes

    def handler(request: httpx.Request) -> httpx.Response:
        range_header = request.headers.get("Range")
        assert range_header == "bytes=5-"
        remaining = full_content[5:]
        return httpx.Response(
            206,
            content=remaining,
            headers={
                "Content-Length": str(len(remaining)),
                "Content-Range": f"bytes 5-15/{len(full_content)}",
            },
        )

    transport = httpx.MockTransport(handler)
    client = httpx.Client(transport=transport)
    dest = tmp_path / "estabelecimentos.zip"
    part_file = dest.parent / f"{dest.name}.part"
    part_file.write_bytes(part_initial_content)

    result = download_file(
        url="https://example.com/estabelecimentos.zip",
        destination=dest,
        client=client,
        resume=True,
    )

    assert result == dest
    assert dest.read_bytes() == full_content
    assert not part_file.exists()


def test_download_file_already_completed(tmp_path: Path) -> None:
    """Evita rebaixar se o arquivo final já existir com tamanho idêntico ao Content-Length."""
    content: bytes = b"arquivo_existente_valido"

    def handler(request: httpx.Request) -> httpx.Response:
        if request.method == "HEAD":
            return httpx.Response(200, headers={"Content-Length": str(len(content))})
        return httpx.Response(200, content=content)

    transport = httpx.MockTransport(handler)
    client = httpx.Client(transport=transport)
    dest = tmp_path / "socios.zip"
    dest.write_bytes(content)

    result = download_file(
        url="https://example.com/socios.zip",
        destination=dest,
        client=client,
    )
    assert result == dest
    assert dest.read_bytes() == content


def test_download_file_truncated_existing_file_forces_redownload(tmp_path: Path) -> None:
    """Prevenção de bug: Arquivo pré-existente truncado (tamanho diverge) deve ser baixado integralmente."""
    full_content: bytes = b"conteudo_completo_de_100_bytes_para_validar_tamanho"
    truncated_content: bytes = b"conteudo_cortado"

    def handler(request: httpx.Request) -> httpx.Response:
        if request.method == "HEAD":
            return httpx.Response(200, headers={"Content-Length": str(len(full_content))})
        return httpx.Response(200, content=full_content)

    transport = httpx.MockTransport(handler)
    client = httpx.Client(transport=transport)
    dest = tmp_path / "estabelecimentos_truncado.zip"
    dest.write_bytes(truncated_content)

    result = download_file(
        url="https://example.com/estabelecimentos_truncado.zip",
        destination=dest,
        client=client,
    )

    assert result == dest
    assert dest.read_bytes() == full_content
    assert dest.stat().st_size == len(full_content)


def test_download_file_head_error_gracefully_falls_back_to_get(tmp_path: Path) -> None:
    """Cenário não-corriqueiro: Falha no HEAD check (ex: 405 Method Not Allowed ou timeout) prossegue para GET."""
    content: bytes = b"dados_baixados_via_fallback"

    def handler(request: httpx.Request) -> httpx.Response:
        if request.method == "HEAD":
            raise httpx.RequestError("HEAD not allowed", request=request)
        return httpx.Response(200, content=content, headers={"Content-Length": str(len(content))})

    transport = httpx.MockTransport(handler)
    client = httpx.Client(transport=transport)
    dest = tmp_path / "fallback.zip"
    # Cria arquivo pré-existente que não pôde ser conferido por HEAD
    dest.write_bytes(b"antigo")

    result = download_file(
        url="https://example.com/fallback.zip",
        destination=dest,
        client=client,
    )

    assert result == dest
    assert dest.read_bytes() == content


def test_download_file_http_error_raises_and_cleans_up(tmp_path: Path) -> None:
    """Garante que códigos de erro HTTP levantem exceção apropriada sem criar arquivo final."""

    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(404, text="Not Found")

    transport = httpx.MockTransport(handler)
    client = httpx.Client(transport=transport)
    dest = tmp_path / "inexistente.zip"

    with pytest.raises(httpx.HTTPStatusError):
        download_file(
            url="https://example.com/inexistente.zip",
            destination=dest,
            client=client,
        )
    assert not dest.exists()


def test_download_file_server_ignores_range_returns_200(tmp_path: Path) -> None:
    """Valida comportamento quando servidor ignora Range e responde 200 OK reiniciando o arquivo."""
    full_content: bytes = b"conteudo_completo_novo"

    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(200, content=full_content)

    transport = httpx.MockTransport(handler)
    client = httpx.Client(transport=transport)
    dest = tmp_path / "teste_no_range.zip"
    part_file = dest.parent / f"{dest.name}.part"
    part_file.write_bytes(b"lixo_antigo")

    result = download_file(
        url="https://example.com/teste_no_range.zip",
        destination=dest,
        client=client,
        resume=True,
    )

    assert result == dest
    assert dest.read_bytes() == full_content


def test_download_files_concurrently(tmp_path: Path) -> None:
    """Valida download de múltiplos arquivos em paralelo através de threads."""
    files_map = {
        "https://example.com/f1.zip": b"conteudo_1",
        "https://example.com/f2.zip": b"conteudo_2",
        "https://example.com/f3.zip": b"conteudo_3",
    }

    def handler(request: httpx.Request) -> httpx.Response:
        url_str = str(request.url)
        content = files_map[url_str]
        return httpx.Response(
            200, content=content, headers={"Content-Length": str(len(content))}
        )

    transport = httpx.MockTransport(handler)
    client = httpx.Client(transport=transport)
    items = [
        ("https://example.com/f1.zip", tmp_path / "f1.zip"),
        ("https://example.com/f2.zip", tmp_path / "f2.zip"),
        ("https://example.com/f3.zip", tmp_path / "f3.zip"),
    ]

    downloaded = download_files_concurrently(
        items=items,
        client=client,
        max_workers=3,
    )

    assert len(downloaded) == 3
    for url, path in items:
        assert path.is_file()
        assert path.read_bytes() == files_map[url]


def test_download_files_concurrently_handles_worker_error(tmp_path: Path) -> None:
    """Valida que falha em worker de download concorrente propaga a exceção."""

    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(500, text="Internal Server Error")

    transport = httpx.MockTransport(handler)
    client = httpx.Client(transport=transport)
    dest = tmp_path / "fail.zip"

    with pytest.raises(httpx.HTTPStatusError):
        download_files_concurrently(
            items=[("https://example.com/fail.zip", dest)],
            client=client,
            max_workers=1,
        )
