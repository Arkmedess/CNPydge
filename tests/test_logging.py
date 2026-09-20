"""Suíte de testes para o sistema de logging padronizado do CNPydge."""

import logging

import pytest
from cnpydge.logging import get_logger, setup_logging


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
