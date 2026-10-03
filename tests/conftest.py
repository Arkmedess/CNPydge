"""Configuração global de fixtures e sanitização para a suíte de testes do CNPydge."""

import logging
from collections.abc import Generator

import pytest
from cnpydge.logging import DEFAULT_LOGGER_NAME


@pytest.fixture(autouse=True)
def reset_cnpydge_logging() -> Generator[None]:
    """Garante isolamento absoluto entre testes, limpando handlers mutados e restaurando NullHandler.

    Elimina ruídos de log e previne vazamento de estado (State Contamination)
    quando testes anteriores configuram handlers globais.
    """
    yield
    logger = logging.getLogger(DEFAULT_LOGGER_NAME)
    for handler in list(logger.handlers):
        if not isinstance(handler, logging.NullHandler):
            logger.removeHandler(handler)
    logger.setLevel(logging.INFO)


@pytest.fixture
def sample_empresa_csv_content() -> str:
    """Retorna linha sintética representativa de Empresas com acentuação e caracteres especiais."""
    return (
        '"12345678";"AÇOUGUE & MERCEARIA SÃO JOSÉ LTDA";"2062";"49";"150000,50";"03";""\n'
        '"87654321";"INDÚSTRIA E COMÉRCIO ESPERANÇA S.A.";"2054";"10";"5000000,00";"05";"BRASILIA"\n'
    )


@pytest.fixture
def sample_socio_csv_content() -> str:
    """Retorna linha sintética representativa de Sócios com CPF mascarado e dados de qualificação."""
    return '"12345678";"2";"JOÃO DA SILVA CONCEIÇÃO";"***123456**";"49";"20210615";"";"***000000**";"MARIA SILVA";"05";"5"\n'


@pytest.fixture
def sample_estabelecimento_csv_content() -> str:
    """Retorna linha sintética de Estabelecimentos com endereçamento e contatos."""
    return (
        '"12345678";"0001";"95";"1";"MATRIZ";"02";"20210510";"00";"";"";"20210510";"6201501";"";'
        '"AVENIDA";"PAULISTA";"1000";"SALA 10";"BELA VISTA";"01310100";"SP";"7107";"11";"33334444";"";"";"";"";"contato@empresa.com";"";""\n'
    )


@pytest.fixture
def sample_simples_csv_content() -> str:
    """Retorna linha sintética de Simples Nacional / MEI."""
    return '"12345678";"S";"20200101";"20221231";"N";"";""\n'


@pytest.fixture
def sample_cnae_csv_content() -> str:
    """Retorna linha sintética de Domínio CNAE."""
    return '"6201501";"DESENVOLVIMENTO DE PROGRAMAS DE COMPUTADOR SOB ENCOMENDA"\n'


@pytest.fixture
def sample_municipio_csv_content() -> str:
    """Retorna linha sintética de Domínio Município."""
    return '"7107";"SÃO PAULO"\n'
