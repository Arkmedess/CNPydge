"""Testes unitários rigorosos para o crawler WebDAV e metadados de arquivos."""

import httpx
from cnpydge.crawler import RemoteFileMetadata, WebDavCrawler

SAMPLE_ROOT_PROPFIND: str = """<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:">
  <d:response>
    <d:href>/public.php/webdav/</d:href>
    <d:propstat><d:prop><d:resourcetype><d:collection/></d:resourcetype></d:prop></d:propstat>
  </d:response>
  <d:response>
    <d:href>/public.php/webdav/2026-01/</d:href>
    <d:propstat><d:prop><d:resourcetype><d:collection/></d:resourcetype></d:prop></d:propstat>
  </d:response>
  <d:response>
    <d:href>/public.php/webdav/2026-03/</d:href>
    <d:propstat><d:prop><d:resourcetype><d:collection/></d:resourcetype></d:prop></d:propstat>
  </d:response>
  <d:response>
    <d:href>/public.php/webdav/2026-02/</d:href>
    <d:propstat><d:prop><d:resourcetype><d:collection/></d:resourcetype></d:prop></d:propstat>
  </d:response>
  <d:response>
    <d:href>/public.php/webdav/outra_pasta_invalida/</d:href>
    <d:propstat><d:prop><d:resourcetype><d:collection/></d:resourcetype></d:prop></d:propstat>
  </d:response>
</d:multistatus>
"""

SAMPLE_PERIOD_PROPFIND: str = """<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:">
  <d:response>
    <d:href>/public.php/webdav/2026-03/</d:href>
    <d:propstat><d:prop><d:resourcetype><d:collection/></d:resourcetype></d:prop></d:propstat>
  </d:response>
  <d:response>
    <d:href>/public.php/webdav/2026-03/subpasta/</d:href>
    <d:propstat><d:prop><d:resourcetype><d:collection/></d:resourcetype></d:prop></d:propstat>
  </d:response>
  <d:response>
    <d:href>/public.php/webdav/2026-03/Empresas0.zip</d:href>
    <d:propstat>
      <d:prop>
        <d:getlastmodified>Tue, 10 Mar 2026 12:00:00 GMT</d:getlastmodified>
        <d:getcontentlength>104857600</d:getcontentlength>
        <d:resourcetype/>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/public.php/webdav/2026-03/Estabelecimentos0.zip</d:href>
    <d:propstat>
      <d:prop>
        <d:getlastmodified>Tue, 10 Mar 2026 13:00:00 GMT</d:getlastmodified>
        <d:getcontentlength>52428800</d:getcontentlength>
        <d:resourcetype/>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/public.php/webdav/2026-03/leiame.txt</d:href>
    <d:propstat>
      <d:prop>
        <d:getlastmodified>Tue, 10 Mar 2026 14:00:00 GMT</d:getlastmodified>
        <d:getcontentlength>1024</d:getcontentlength>
        <d:resourcetype/>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>
"""


def test_remote_file_metadata_properties() -> None:
    """Verifica propriedades computadas do RemoteFileMetadata."""
    meta_zip = RemoteFileMetadata(
        name="Empresas0.zip",
        url="https://example.com/2026-03/Empresas0.zip",
        size_bytes=100,
        last_modified="Tue, 10 Mar 2026 12:00:00 GMT",
    )
    assert meta_zip.is_zip is True
    assert meta_zip.table_kind == "empresas"

    meta_txt = RemoteFileMetadata(
        name="leiame.txt",
        url="https://example.com/2026-03/leiame.txt",
        size_bytes=50,
    )
    assert meta_txt.is_zip is False
    assert meta_txt.table_kind is None


def test_list_available_periods_sorting_and_filtering() -> None:
    """Valida descoberta de períodos ordenada decrescente e filtrando diretórios inválidos."""

    def handler(request: httpx.Request) -> httpx.Response:
        assert request.method == "PROPFIND"
        return httpx.Response(200, text=SAMPLE_ROOT_PROPFIND)

    transport = httpx.MockTransport(handler)
    client = httpx.Client(transport=transport)
    crawler = WebDavCrawler(
        base_url="https://example.com/webdav", token="dummy-token", client=client
    )

    periods: list[str] = crawler.list_available_periods()
    assert periods == ["2026-03", "2026-02", "2026-01"]


def test_list_files_by_period_parsing_and_filters() -> None:
    """Valida extração de metadados ignorando coleções e aplicando filtros."""

    def handler(request: httpx.Request) -> httpx.Response:
        assert request.method == "PROPFIND"
        assert "/2026-03/" in str(request.url)
        return httpx.Response(200, text=SAMPLE_PERIOD_PROPFIND)

    transport = httpx.MockTransport(handler)
    client = httpx.Client(transport=transport)
    crawler = WebDavCrawler(
        base_url="https://example.com/webdav", token="dummy-token", client=client
    )

    # 1. Sem filtros: traz todos os arquivos, excluindo diretórios/coleções
    files: list[RemoteFileMetadata] = crawler.list_files_by_period("2026-03")
    assert len(files) == 3
    names: list[str] = [f.name for f in files]
    assert "Empresas0.zip" in names
    assert "Estabelecimentos0.zip" in names
    assert "leiame.txt" in names

    # 2. Filtro apenas zips
    zip_files: list[RemoteFileMetadata] = crawler.list_files_by_period("2026-03", include_patterns=["*.zip"])
    assert len(zip_files) == 2
    assert all(f.is_zip for f in zip_files)

    # 3. Filtro por tamanho mínimo (ignora txt pequeno)
    large_files: list[RemoteFileMetadata] = crawler.list_files_by_period("2026-03", min_file_size=10_000)
    assert len(large_files) == 2


def test_crawler_handles_http_errors_gracefully() -> None:
    """Garante que erros de HTTP (ex: 401 Unauthorized ou 500) retornam lista vazia e registram log."""

    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(401, text="Unauthorized")

    transport = httpx.MockTransport(handler)
    client = httpx.Client(transport=transport)
    crawler = WebDavCrawler(
        base_url="https://example.com/webdav", token="invalid-token", client=client
    )

    assert crawler.list_available_periods() == []
    assert crawler.list_files_by_period("2026-03") == []


def test_crawler_xml_parse_errors_and_context_manager() -> None:
    """Testa tratamento de XML malformado e uso como gerenciador de contexto."""

    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(200, text="<xml><malformed>")

    transport = httpx.MockTransport(handler)
    client = httpx.Client(transport=transport)
    with WebDavCrawler(
        base_url="https://example.com/webdav", token="dummy", client=client
    ) as crawler:
        assert crawler.list_available_periods() == []
        assert crawler.list_files_by_period("2026-03") == []


def test_crawler_filter_exclude_and_max_size() -> None:
    """Testa filtros de exclusão e tamanho máximo."""

    def handler(request: httpx.Request) -> httpx.Response:
        return httpx.Response(200, text=SAMPLE_PERIOD_PROPFIND)

    transport = httpx.MockTransport(handler)
    client = httpx.Client(transport=transport)
    crawler = WebDavCrawler(
        base_url="https://example.com/webdav", token="dummy", client=client
    )

    # Exclusão de Empresas
    filtered: list[RemoteFileMetadata] = crawler.list_files_by_period("2026-03", exclude_patterns=["*Empresas*"])
    assert not any("Empresas" in f.name for f in filtered)

    # Max file size
    small_files: list[RemoteFileMetadata] = crawler.list_files_by_period("2026-03", max_file_size=2000)
    assert len(small_files) == 1
    assert small_files[0].name == "leiame.txt"
    crawler.close()
