"""Ponto de entrada de demonstração e teste do CNPydge."""

import tempfile
from pathlib import Path

import cnpydge


def main() -> None:
    """Executa a verificação do motor nativo e da conversão para Parquet."""
    print(f"CNPydge inicializado com sucesso! Versão do motor: {cnpydge.version()}")

    with tempfile.TemporaryDirectory() as tmp_dir:
        csv_file: Path = Path(tmp_dir) / "empresas_exemplo.csv"
        parquet_file: Path = Path(tmp_dir) / "empresas_exemplo.parquet"

        amostra_csv: str = (
            '"12345678";"EMPRESA ALFA LTDA";"2062";"49";"100000,00";"03";""\n'
            '"87654321";"EMPRESA BETA S.A.";"2054";"10";"5000000,50";"05";"BRASILIA"\n'
            '"12ABC345";"STARTUP INOVADORA LTDA";"2062";"49";"150000,00";"01";""\n'
        )
        csv_file.write_text(amostra_csv, encoding="utf-8")

        total_linhas: int = cnpydge.to_parquet(str(csv_file), str(parquet_file))
        tamanho_parquet: int = parquet_file.stat().st_size

        print(
            f"Conversão concluída: {total_linhas} linhas convertidas com sucesso para Parquet "
            f"({tamanho_parquet} bytes no disco)."
        )


if __name__ == "__main__":
    main()
