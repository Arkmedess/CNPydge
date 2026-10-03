"""Infraestrutura de logging padronizado e observabilidade para o CNPydge."""

import json
import logging
import logging.handlers
import os
import re
import sys
from datetime import UTC, datetime
from pathlib import Path
from typing import Any, Final

LOG_FORMAT: Final[str] = (
    "%(asctime)s [%(levelname)s] [%(threadName)s] [%(name)s:%(filename)s:%(lineno)d] %(message)s"
)
DEFAULT_LOGGER_NAME: Final[str] = "cnpydge"

# Expressões regulares para sanitização ativa de dados sensíveis e PII
_URL_CREDENTIALS_REGEX: Final[re.Pattern[str]] = re.compile(
    r"(https?://)([^:\s/@]+):([^@\s]+)@",
    re.IGNORECASE,
)
_BEARER_REGEX: Final[re.Pattern[str]] = re.compile(
    r"(Bearer\s+)[A-Za-z0-9_\-\.]+",
    re.IGNORECASE,
)
_SENSITIVE_KV_REGEX: Final[re.Pattern[str]] = re.compile(
    r"((?:password|passwd|secret|token)\s*[:=]\s*['\"]?)([^'\"\s&]+)(['\"]?)",
    re.IGNORECASE,
)
_CPF_REGEX: Final[re.Pattern[str]] = re.compile(
    r"\b\d{3}\.\d{3}\.\d{3}-\d{2}\b",
)


class SensitiveDataFilter(logging.Filter):
    """Filtro de interceptação para mascaramento ativo de segredos, credenciais e PII."""

    @staticmethod
    def sanitize(text: str) -> str:
        """Aplica expressões regulares de mascaramento a strings de texto.

        Args:
            text: Conteúdo textual a ser auditado e sanitizado.

        Returns:
            String com credenciais, senhas e CPFs mascarados.
        """
        text = _URL_CREDENTIALS_REGEX.sub(r"\1\2:***@", text)
        text = _BEARER_REGEX.sub(r"\1***", text)
        text = _SENSITIVE_KV_REGEX.sub(r"\1***\3", text)
        text = _CPF_REGEX.sub(r"***.***.***-**", text)
        return text

    def filter(self, record: logging.LogRecord) -> bool:
        """Intercepta o registro antes do despache aos handlers para sanitização.

        Args:
            record: Registro de log emitido pelo logger.

        Returns:
            Sempre True para permitir o fluxo após a higienização do conteúdo.
        """
        if record.args:
            try:
                formatted = record.getMessage()
                record.msg = self.sanitize(formatted)
                record.args = ()
            except (TypeError, ValueError):
                if isinstance(record.msg, str):
                    record.msg = self.sanitize(record.msg)
        elif isinstance(record.msg, str):
            record.msg = self.sanitize(record.msg)

        return True


class IsoUtcFormatter(logging.Formatter):
    """Formatador de log com timestamp em conformidade com RFC 3339 / ISO 8601 UTC."""

    def formatTime(self, record: logging.LogRecord, datefmt: str | None = None) -> str:
        """Gera timestamp UTC no formato RFC 3339 (ex.: 2026-10-03T20:15:30.123Z).

        Args:
            record: Registro do log em processamento.
            datefmt: Formato opcional de data (ignorado em favor de UTC ISO 8601).

        Returns:
            String com timestamp canônico em UTC com sufixo 'Z'.
        """
        dt = datetime.fromtimestamp(record.created, tz=UTC)
        return dt.strftime("%Y-%m-%dT%H:%M:%S.") + f"{int(record.msecs):03d}Z"


class JsonLogFormatter(logging.Formatter):
    """Formatador para emissão de eventos em JSON estruturado para observabilidade."""

    def format(self, record: logging.LogRecord) -> str:
        """Serializa o evento de log em formato JSON para ingestão em collectors.

        Args:
            record: Registro de log a ser formatado.

        Returns:
            Linha serializada em JSON com atributos estruturados.
        """
        dt = datetime.fromtimestamp(record.created, tz=UTC)
        timestamp_str = dt.strftime("%Y-%m-%dT%H:%M:%S.") + f"{int(record.msecs):03d}Z"

        payload: dict[str, Any] = {
            "timestamp": timestamp_str,
            "level": record.levelname,
            "logger": record.name,
            "thread": record.threadName,
            "module": record.module,
            "filename": record.filename,
            "line": record.lineno,
            "message": record.getMessage(),
        }

        if record.exc_info:
            payload["exception"] = self.formatException(record.exc_info)

        return json.dumps(payload, ensure_ascii=False)


def get_logger(name: str = DEFAULT_LOGGER_NAME) -> logging.Logger:
    """Retorna um logger configurado para o componente especificado.

    Args:
        name: Identificador do logger (padrão: 'cnpydge').

    Returns:
        Instância de logging.Logger correspondente.
    """
    return logging.getLogger(name)


def setup_logging(
    level: int | str | None = None,
    handler: logging.Handler | None = None,
    json_format: bool | None = None,
    log_to_file: bool = False,
    log_dir: str | Path | None = None,
) -> logging.Logger:
    """Configura handlers, sanitização e formato padronizado para o logger raiz do CNPydge.

    Args:
        level: Nível de severidade (DEBUG, INFO, etc.). Se None, lê CNPYDGE_LOG_LEVEL.
        handler: Handler customizado opcional. Se None, cria StreamHandler para stderr.
        json_format: Se True, emite JSON. Se None, verifica CNPYDGE_LOG_FORMAT=json.
        log_to_file: Se True, anexa RotatingFileHandler para gravação em disco.
        log_dir: Diretório para arquivos de log quando log_to_file=True.

    Returns:
        Logger raiz configurado de forma idempotente para o CNPydge.
    """
    logger = get_logger(DEFAULT_LOGGER_NAME)

    if level is None:
        raw_level: str = os.getenv("CNPYDGE_LOG_LEVEL", "INFO").upper()
        level = getattr(logging, raw_level, logging.INFO)
    elif isinstance(level, str):
        level = getattr(logging, level.upper(), logging.INFO)

    logger.setLevel(level)

    if json_format is None:
        raw_format = os.getenv("CNPYDGE_LOG_FORMAT", "").lower()
        json_format = raw_format in ("json", "1", "true")

    formatter: logging.Formatter = (
        JsonLogFormatter() if json_format else IsoUtcFormatter(LOG_FORMAT)
    )
    filter_instance = SensitiveDataFilter()

    # Limpeza defensiva de handlers ativos (preservando NullHandler se houver) para idempotência
    for existing_handler in list(logger.handlers):
        if not isinstance(existing_handler, logging.NullHandler):
            logger.removeHandler(existing_handler)

    target_handler = handler or logging.StreamHandler(sys.stderr)
    target_handler.setFormatter(formatter)
    target_handler.addFilter(filter_instance)
    logger.addHandler(target_handler)

    if log_to_file:
        output_dir = Path(log_dir or "logs")
        output_dir.mkdir(parents=True, exist_ok=True)
        file_handler = logging.handlers.RotatingFileHandler(
            output_dir / "cnpydge.log",
            maxBytes=10 * 1024 * 1024,
            backupCount=5,
            encoding="utf-8",
        )
        file_handler.setFormatter(formatter)
        file_handler.addFilter(filter_instance)
        logger.addHandler(file_handler)

    return logger


# Registro de NullHandler na inicialização da biblioteca conforme PEP 282
logging.getLogger(DEFAULT_LOGGER_NAME).addHandler(logging.NullHandler())
