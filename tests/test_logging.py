"""Suíte de testes para o sistema de logging padronizado do CNPydge."""

import io
import json
import logging
from pathlib import Path

import pytest
from cnpydge.logging import (
    DEFAULT_LOGGER_NAME,
    IsoUtcFormatter,
    JsonLogFormatter,
    SensitiveDataFilter,
    get_logger,
    setup_logging,
)


def test_setup_logging_level_configuration() -> None:
    """Verifica se setup_logging configura o nível e os handlers corretamente."""
    logger = setup_logging(level=logging.DEBUG)
    assert logger.level == logging.DEBUG
    assert len(logger.handlers) > 0


def test_setup_logging_env_var(monkeypatch: pytest.MonkeyPatch) -> None:
    """Valida a resolução do nível de log através de variável de ambiente."""
    monkeypatch.setenv("CNPYDGE_LOG_LEVEL", "WARNING")
    logger = setup_logging()
    assert logger.level == logging.WARNING


def test_get_logger_hierarchy() -> None:
    """Garante que instâncias filhas pertençam à hierarquia 'cnpydge'."""
    sub_logger = get_logger("cnpydge.converter")
    assert sub_logger.name == "cnpydge.converter"


def test_sensitive_data_filter_masks_credentials_in_url() -> None:
    """Garante que credenciais básicas em URLs (user:pass@host) sejam ofuscadas."""
    filt = SensitiveDataFilter()
    record = logging.LogRecord(
        name="cnpydge.test",
        level=logging.INFO,
        pathname="",
        lineno=0,
        msg="Conectando em https://admin:secreto123@webdav.rfb.gov.br/caminho",
        args=(),
        exc_info=None,
    )
    filt.filter(record)
    assert "secreto123" not in record.msg
    assert "https://admin:***@webdav.rfb.gov.br/caminho" in record.msg


def test_sensitive_data_filter_masks_args_and_bearer() -> None:
    """Valida o mascaramento de segredos passados via args e tokens Bearer."""
    filt = SensitiveDataFilter()
    record = logging.LogRecord(
        name="cnpydge.test",
        level=logging.INFO,
        pathname="",
        lineno=0,
        msg="Falha ao autenticar com token=%s e Bearer secret_token_abc",
        args=("senha_secreta_99",),
        exc_info=None,
    )
    filt.filter(record)
    assert "senha_secreta_99" not in record.msg
    assert "token=***" in record.msg
    assert "secret_token_abc" not in record.msg
    assert "Bearer ***" in record.msg


def test_sensitive_data_filter_masks_cpf() -> None:
    """Garante que sequências de CPF com pontuação sejam mascaradas para a LGPD."""
    filt = SensitiveDataFilter()
    record = logging.LogRecord(
        name="cnpydge.test",
        level=logging.INFO,
        pathname="",
        lineno=0,
        msg="Sócio cadastrado com CPF 123.456.789-00 registrado.",
        args=(),
        exc_info=None,
    )
    filt.filter(record)
    assert "123.456.789-00" not in record.msg
    assert "***.***.***-**" in record.msg


def test_iso_utc_formatter_produces_utc_z() -> None:
    """Valida se o IsoUtcFormatter gera timestamps UTC terminados em 'Z'."""
    formatter = IsoUtcFormatter()
    record = logging.LogRecord(
        name="cnpydge.test",
        level=logging.INFO,
        pathname="teste.py",
        lineno=10,
        msg="Mensagem de teste",
        args=(),
        exc_info=None,
    )
    formatted_time = formatter.formatTime(record)
    assert formatted_time.endswith("Z")
    assert "T" in formatted_time


def test_json_log_formatter_with_exception() -> None:
    """Verifica a serialização de exceção e traceback no JsonLogFormatter."""
    formatter = JsonLogFormatter()
    try:
        raise ValueError("Erro de teste simulado")
    except ValueError:
        import sys

        exc_info = sys.exc_info()

    record = logging.LogRecord(
        name="cnpydge.test",
        level=logging.ERROR,
        pathname="teste.py",
        lineno=10,
        msg="Falha capturada",
        args=(),
        exc_info=exc_info,
    )
    output = formatter.format(record)
    payload = json.loads(output)
    assert payload["level"] == "ERROR"
    assert "exception" in payload
    assert "ValueError: Erro de teste simulado" in payload["exception"]


def test_setup_logging_json_format_emits_valid_json() -> None:
    """Verifica se o modo json_format emite registros válidos em JSON estruturado."""
    stream = io.StringIO()
    handler = logging.StreamHandler(stream)
    logger = setup_logging(level=logging.INFO, handler=handler, json_format=True)
    logger.info("Teste de evento estruturado")
    output = stream.getvalue().strip()
    payload = json.loads(output)
    assert payload["level"] == "INFO"
    assert payload["message"] == "Teste de evento estruturado"
    assert payload["logger"] == DEFAULT_LOGGER_NAME
    assert payload["timestamp"].endswith("Z")
    assert "thread" in payload


def test_setup_logging_reconfiguration_idempotency() -> None:
    """Garante que chamadas sucessivas reconfigurem handlers sem duplicá-los."""
    stream = io.StringIO()
    handler1 = logging.StreamHandler(stream)
    logger = setup_logging(level=logging.INFO, handler=handler1)
    non_null_handlers_initial = [
        h for h in logger.handlers if not isinstance(h, logging.NullHandler)
    ]
    assert len(non_null_handlers_initial) == 1

    handler2 = logging.StreamHandler(stream)
    logger2 = setup_logging(level=logging.DEBUG, handler=handler2, json_format=False)
    assert logger2.level == logging.DEBUG
    non_null_handlers_after = [
        h for h in logger2.handlers if not isinstance(h, logging.NullHandler)
    ]
    assert len(non_null_handlers_after) == 1
    assert non_null_handlers_after[0] is handler2


def test_setup_logging_file_handler(tmp_path: Path) -> None:
    """Verifica a criação de arquivo de log com RotatingFileHandler."""
    log_dir = tmp_path / "logs"
    logger = setup_logging(level=logging.INFO, log_to_file=True, log_dir=log_dir)
    logger.info("Registro persistido em arquivo")

    log_file = log_dir / "cnpydge.log"
    assert log_file.is_file()
    assert "Registro persistido em arquivo" in log_file.read_text(encoding="utf-8")
