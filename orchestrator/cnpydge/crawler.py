"""Descoberta e catalogação de arquivos de dados abertos de CNPJ via WebDAV."""

import fnmatch
import os
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from datetime import datetime
from types import TracebackType
from typing import Final, Self

import httpx

from cnpydge.logging import get_logger

LOGGER = get_logger("cnpydge.crawler")

DAV_NS: Final[dict[str, str]] = {
    "d": "DAV:",
    "oc": "http://owncloud.org/ns",
    "nc": "http://nextcloud.org/ns",
}

TABLE_PREFIX_MAP: Final[dict[str, str]] = {
    "empresa": "empresas",
    "estabele": "estabelecimentos",
    "socio": "socios",
    "simples": "simples",
    "cnae": "cnaes",
    "motivo": "motivos",
    "municipio": "municipios",
    "natureza": "naturezas",
    "pais": "paises",
    "qualificacao": "qualificacoes",
}


@dataclass(frozen=True, slots=True)
class RemoteFileMetadata:
    """Metadados de arquivo público identificado no repositório WebDAV.

    Attributes:
        name: Nome do arquivo (ex.: 'Empresas0.zip').
        url: URL completa para download do recurso.
        size_bytes: Tamanho do arquivo em bytes.
        last_modified: Data/hora da última modificação reportada pelo servidor.
    """

    name: str
    url: str
    size_bytes: int
    last_modified: str | None = None

    @property
    def is_zip(self) -> bool:
        """Verifica se o recurso remoto é um arquivo compactado .zip.

        Returns:
            True se a extensão for '.zip' (case-insensitive), False caso contrário.
        """
        return self.name.lower().endswith(".zip")

    @property
    def table_kind(self) -> str | None:
        """Infere a tabela correspondente com base no prefixo do nome do arquivo.

        Returns:
            Identificador canônico da tabela (ex.: 'empresas', 'socios') ou None se não mapeado.
        """
        lower_name: str = self.name.lower()
        for prefix, table in TABLE_PREFIX_MAP.items():
            if prefix in lower_name:
                return table
        return None


class WebDavCrawler:
    """Serviço de descoberta e inspeção de arquivos em repositórios WebDAV/Nextcloud."""

    def __init__(
        self,
        base_url: str | None = None,
        token: str | None = None,
        client: httpx.Client | None = None,
    ) -> None:
        """Inicializa o crawler com autenticação por token público.

        Args:
            base_url: URL base do endpoint WebDAV (ex: /public.php/webdav).
            token: Token de compartilhamento público.
            client: Instância opcional de httpx.Client (para injeção em testes).
        """
        self.base_url: str = (base_url or os.getenv("CNPYDGE_WEBDAV_URL", "")).rstrip("/")
        self.token: str = token or os.getenv("CNPYDGE_WEBDAV_TOKEN", "")
        self.auth: tuple[str, str] | None = (self.token, "") if self.token else None

        self._managed_client: bool = client is None
        self.client: httpx.Client = client or httpx.Client(timeout=60.0)
        LOGGER.debug("WebDavCrawler inicializado para base_url='%s'", self.base_url)

    def list_available_periods(self) -> list[str]:
        """Descobre todos os períodos (pastas YYYY-MM) disponíveis no repositório.

        Returns:
            Lista de períodos ordenados em ordem cronológica decrescente.
        """
        url = f"{self.base_url}/"
        LOGGER.info("Buscando períodos disponíveis no WebDAV: %s", url)

        try:
            response = self.client.request(
                "PROPFIND",
                url,
                auth=self.auth,
                headers={"Depth": "1"},
            )
            response.raise_for_status()
        except (httpx.HTTPStatusError, httpx.RequestError) as err:
            LOGGER.error("Falha ao consultar períodos no WebDAV (%s): %s", url, err)
            return []

        try:
            root = ET.fromstring(response.text)
        except ET.ParseError as err:
            LOGGER.error("Falha de parsing XML ao listar períodos: %s", err)
            return []

        periods: list[str] = []
        for href_elem in root.findall(".//d:href", DAV_NS):
            href: str = href_elem.text or ""
            parts: list[str] = href.strip("/").split("/")
            if not parts:
                continue
            folder_name: str = parts[-1]
            if len(folder_name) == 7 and folder_name[4] == "-":
                try:
                    datetime.strptime(f"{folder_name}-01", "%Y-%m-%d").replace(
                        tzinfo=datetime.now().astimezone().tzinfo
                    )
                    periods.append(folder_name)
                except ValueError:
                    continue

        periods.sort(reverse=True)
        LOGGER.info("Períodos identificados (%d): %s", len(periods), periods)
        return periods

    def list_files_by_period(
        self,
        period: str,
        include_patterns: list[str] | None = None,
        exclude_patterns: list[str] | None = None,
        min_file_size: int | None = None,
        max_file_size: int | None = None,
    ) -> list[RemoteFileMetadata]:
        """Lista os arquivos disponíveis para um período específico aplicando filtros.

        Args:
            period: Período no formato YYYY-MM (ex.: '2026-03').
            include_patterns: Padrões glob para inclusão (ex.: ['*.zip']).
            exclude_patterns: Padrões glob para exclusão.
            min_file_size: Tamanho mínimo em bytes.
            max_file_size: Tamanho máximo em bytes.

        Returns:
            Lista de metadados dos arquivos validados.
        """
        url = f"{self.base_url}/{period}/"
        LOGGER.info("Listando arquivos WebDAV para o período '%s' em: %s", period, url)

        try:
            response = self.client.request(
                "PROPFIND",
                url,
                auth=self.auth,
                headers={"Depth": "1"},
            )
            response.raise_for_status()
        except (httpx.HTTPStatusError, httpx.RequestError) as err:
            LOGGER.error("Falha ao consultar arquivos do período '%s': %s", period, err)
            return []

        file_list: list[RemoteFileMetadata] = self._parse_webdav_response(response.text, period)
        return self._apply_filters(
            file_list,
            include_patterns=include_patterns,
            exclude_patterns=exclude_patterns,
            min_file_size=min_file_size,
            max_file_size=max_file_size,
        )

    def _parse_webdav_response(
        self,
        xml_content: str,
        period: str,
    ) -> list[RemoteFileMetadata]:
        """Processa a resposta XML de um PROPFIND e extrai metadados dos arquivos.

        Args:
            xml_content: Conteúdo textual XML recebido na resposta HTTP multistatus.
            period: Subdiretório/período correspondente (ex.: '2026-03').

        Returns:
            Lista de instâncias imutáveis de RemoteFileMetadata dos arquivos válidos.
        """
        files: list[RemoteFileMetadata] = []
        try:
            root: ET.Element = ET.fromstring(xml_content)
        except ET.ParseError as err:
            LOGGER.error("XML inválido recebido do WebDAV: %s", err)
            return []

        for resp in root.findall("d:response", DAV_NS):
            href: str | None = resp.findtext("d:href", namespaces=DAV_NS)
            if not href:
                continue

            name: str = href.strip("/").split("/")[-1]
            resource_type: ET.Element | None = resp.find(".//d:resourcetype", DAV_NS)
            is_collection: bool = (
                resource_type is not None
                and resource_type.find("d:collection", DAV_NS) is not None
            )

            # Ignora pastas/coleções e entradas vazias
            if not name or is_collection:
                continue

            last_modified: str | None = resp.findtext(".//d:getlastmodified", namespaces=DAV_NS)
            size_bytes_raw: str | None = resp.findtext(".//d:getcontentlength", namespaces=DAV_NS)
            size_bytes: int = (
                int(size_bytes_raw)
                if size_bytes_raw and size_bytes_raw.isdigit()
                else 0
            )

            file_url = f"{self.base_url}/{period}/{name}"
            files.append(
                RemoteFileMetadata(
                    name=name,
                    url=file_url,
                    size_bytes=size_bytes,
                    last_modified=last_modified,
                )
            )

        return files

    @staticmethod
    def _apply_filters(
        files: list[RemoteFileMetadata],
        include_patterns: list[str] | None = None,
        exclude_patterns: list[str] | None = None,
        min_file_size: int | None = None,
        max_file_size: int | None = None,
    ) -> list[RemoteFileMetadata]:
        """Aplica filtros de tamanho e padrões glob à lista de arquivos.

        Args:
            files: Lista de metadados dos arquivos remotos a filtrar.
            include_patterns: Padrões glob inclusivos (ex.: ['*.zip']).
            exclude_patterns: Padrões glob exclusivos para descartar.
            min_file_size: Tamanho mínimo em bytes (inclusive).
            max_file_size: Tamanho máximo em bytes (inclusive).

        Returns:
            Lista filtrada de RemoteFileMetadata aderente a todos os critérios.
        """
        result: list[RemoteFileMetadata] = files

        if include_patterns:
            result = [
                f for f in result
                if any(fnmatch.fnmatch(f.name, p) for p in include_patterns)
            ]

        if exclude_patterns:
            result = [
                f for f in result
                if not any(fnmatch.fnmatch(f.name, p) for p in exclude_patterns)
            ]

        if min_file_size is not None:
            result = [f for f in result if f.size_bytes >= min_file_size]

        if max_file_size is not None:
            result = [f for f in result if f.size_bytes <= max_file_size]

        return result

    def close(self) -> None:
        """Encerra conexões abertas caso o client seja gerenciado pela instância."""
        if self._managed_client:
            self.client.close()

    def __enter__(self) -> Self:
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc_val: BaseException | None,
        exc_tb: TracebackType | None,
    ) -> None:
        self.close()
