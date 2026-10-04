"""Ponto de entrada de demonstração e teste do CNPydge."""

import tempfile
from pathlib import Path

import cnpydge
from cnpydge.logging import get_logger, setup_logging

LOGGER = get_logger("cnpydge.main")


DEMO_CASES: list[tuple[str, str, str]] = [
    (
        "empresas",
        "empresas",
        (
            '"12345678";"EMPRESA ALFA LTDA";"2062";"49";"100000,00";"03";""\n'
            '"87654321";"EMPRESA BETA S.A.";"2054";"10";"5000000,50";"05";"BRASILIA"\n'
            '"12ABC345";"STARTUP INOVADORA LTDA";"2062";"49";"150000,00";"01";""\n'
        ),
    ),
    (
        "socios",
        "socios",
        '"12ABC345";"2";"MARIA SILVA";"***123456**";"49";"20200115";"";"***000000**";"JOSE SILVA";"05";"5"\n',
    ),
    (
        "estabelecimentos",
        "estabelecimentos",
        (
            '"12ABC345";"0001";"95";"1";"MATRIZ";"02";"20210510";"00";"";"";"20210510";"6201501";"";'
            '"AVENIDA";"PAULISTA";"1000";"SALA 10";"BELA VISTA";"01310100";"SP";"7107";"11";"33334444";'
            '"";"";"";"";"contato@empresa.com";"";""\n'
        ),
    ),
    (
        "simples",
        "simples",
        '"12ABC345";"S";"20200101";"20211231";"N";"";""\n',
    ),
    (
        "cnaes",
        "cnaes",
        '"6201501";"DESENVOLVIMENTO DE PROGRAMAS DE COMPUTADOR SOB ENCOMENDA"\n',
    ),
    (
        "municipios",
        "municipios",
        '"7107";"SAO PAULO"\n',
    ),
]


def _demo_table(dir_path: Path, name: str, kind: str, sample_csv: str) -> None:
    csv_file = dir_path / f"{name}.csv"
    parquet_file = dir_path / f"{name}.parquet"
    csv_file.write_text(sample_csv, encoding="utf-8")
    rows = cnpydge.to_parquet(str(csv_file), str(parquet_file), kind=kind)
    LOGGER.info(
        "%s: %d linhas convertidas -> %d bytes.",
        name.capitalize(),
        rows,
        parquet_file.stat().st_size,
    )


def main() -> None:
    """Executa a verificação unificada do motor nativo para as tabelas suportadas."""
    setup_logging()
    LOGGER.info("CNPydge inicializado com sucesso! Versão do motor: %s", cnpydge.version())

    with tempfile.TemporaryDirectory() as tmp_dir:
        dir_path = Path(tmp_dir)
        for name, kind, sample_csv in DEMO_CASES:
            _demo_table(dir_path, name, kind, sample_csv)


if __name__ == "__main__":
    main()
