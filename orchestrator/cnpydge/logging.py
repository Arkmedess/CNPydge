"""Infraestrutura de logging padronizado para o CNPydge."""

import logging
import os
import sys
from typing import Final

LOG_FORMAT: Final[str] = (
    "%(asctime)s [%(levelname)s] [%(name)s:%(filename)s:%(lineno)d] %(message)s"
)
DEFAULT_LOGGER_NAME: Final[str] = "cnpydge"


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
) -> logging.Logger:
    """Configura handlers e formato padronizado para o logger raiz do CNPydge.

    Args:
        level: Nível de severidade (DEBUG, INFO, etc.). Se None, lê CNPYDGE_LOG_LEVEL.
        handler: Handler customizado. Se None, cria StreamHandler para stderr.

    Returns:
        Logger raiz configurado para o CNPydge.
    """
    logger = get_logger(DEFAULT_LOGGER_NAME)

    if level is None:
        raw_level: str = os.getenv("CNPYDGE_LOG_LEVEL", "INFO").upper()
        level = getattr(logging, raw_level, logging.INFO)
    elif isinstance(level, str):
        level = getattr(logging, level.upper(), logging.INFO)

    logger.setLevel(level)

    if not logger.handlers:
        target_handler = handler or logging.StreamHandler(sys.stderr)
        target_handler.setFormatter(logging.Formatter(LOG_FORMAT))
        logger.addHandler(target_handler)

    return logger
