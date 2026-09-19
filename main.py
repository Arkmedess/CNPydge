"""Ponto de entrada de demonstração e teste do CNPydge."""

import tempfile
from pathlib import Path

import cnpydge


def main() -> None:
    """Executa a verificação unificada do motor nativo para Empresas, Sócios e Estabelecimentos."""
    print(f"CNPydge inicializado com sucesso! Versão do motor: {cnpydge.version()}")

    with tempfile.TemporaryDirectory() as tmp_dir:
        dir_path: Path = Path(tmp_dir)

        # 1. Teste de Empresas
        emp_csv: Path = dir_path / "empresas.csv"
        emp_parquet: Path = dir_path / "empresas.parquet"
        emp_csv.write_text(
            '"12345678";"EMPRESA ALFA LTDA";"2062";"49";"100000,00";"03";""\n'
            '"87654321";"EMPRESA BETA S.A.";"2054";"10";"5000000,50";"05";"BRASILIA"\n'
            '"12ABC345";"STARTUP INOVADORA LTDA";"2062";"49";"150000,00";"01";""\n',
            encoding="utf-8",
        )
        total_emp: int = cnpydge.to_parquet(
            str(emp_csv), str(emp_parquet), kind="empresas"
        )
        print(
            f"Empresas: {total_emp} linhas convertidas -> {emp_parquet.stat().st_size} bytes."
        )

        # 2. Teste de Sócios
        soc_csv: Path = dir_path / "socios.csv"
        soc_parquet: Path = dir_path / "socios.parquet"
        soc_csv.write_text(
            '"12ABC345";"2";"MARIA SILVA";"***123456**";"49";"20200115";"";"***000000**";"JOSE SILVA";"05";"5"\n',
            encoding="utf-8",
        )
        total_soc: int = cnpydge.to_parquet(
            str(soc_csv), str(soc_parquet), kind="socios"
        )
        print(
            f"Sócios: {total_soc} linhas convertidas -> {soc_parquet.stat().st_size} bytes."
        )

        # 3. Teste de Estabelecimentos
        est_csv: Path = dir_path / "estabelecimentos.csv"
        est_parquet: Path = dir_path / "estabelecimentos.parquet"
        est_csv.write_text(
            '"12ABC345";"0001";"95";"1";"MATRIZ";"02";"20210510";"00";"";"";"20210510";"6201501";"";"AVENIDA";"PAULISTA";"1000";"SALA 10";"BELA VISTA";"01310100";"SP";"7107";"11";"33334444";"";"";"";"";"contato@empresa.com";"";""\n',
            encoding="utf-8",
        )
        total_est: int = cnpydge.to_parquet(
            str(est_csv), str(est_parquet), kind="estabelecimentos"
        )
        print(
            f"Estabelecimentos: {total_est} linhas convertidas -> {est_parquet.stat().st_size} bytes."
        )


if __name__ == "__main__":
    main()
