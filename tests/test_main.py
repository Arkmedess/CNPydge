"""Testes unitários para o módulo executável cnpydge.__main__."""

from pathlib import Path
from unittest.mock import MagicMock

import pytest
from cnpydge.__main__ import main


def test_main_version(capsys: pytest.CaptureFixture[str]) -> None:
    """Valida se --version retorna código 0 e exibe a versão correta."""
    exit_code = main(["--version"])
    captured = capsys.readouterr()

    assert exit_code == 0
    assert "CNPydge v" in captured.out


def test_main_no_args_shows_help(capsys: pytest.CaptureFixture[str]) -> None:
    """Valida se executar sem argumentos exibe a mensagem de ajuda e uso."""
    exit_code = main([])
    captured = capsys.readouterr()

    assert exit_code == 0
    assert "CNPydge - Pipeline de alta vazão" in captured.out
    assert "Comandos disponíveis:" in captured.out


def test_main_run_subcommand(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    """Valida se o subcomando run invoca run_pipeline com os parâmetros corretos."""
    mock_run_pipeline = MagicMock(return_value=[])
    monkeypatch.setattr("cnpydge.run_pipeline", mock_run_pipeline)

    output_dir = tmp_path / "dados"
    exit_code = main(
        [
            "run",
            "--output-dir",
            str(output_dir),
            "--tables",
            "empresas,socios",
            "--profile",
            "balanced_rolling_eviction",
        ]
    )

    assert exit_code == 0
    mock_run_pipeline.assert_called_once_with(
        output_dir=str(output_dir),
        tables=["empresas", "socios"],
        profile="balanced_rolling_eviction",
    )


def test_main_run_handles_exception(
    monkeypatch: pytest.MonkeyPatch,
    tmp_path: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Valida se falhas operacionais retornam código 1 e mensagem em stderr."""
    mock_run_pipeline = MagicMock(side_effect=RuntimeError("Falha de rede simulada"))
    monkeypatch.setattr("cnpydge.run_pipeline", mock_run_pipeline)

    exit_code = main(["run", "--output-dir", str(tmp_path)])
    captured = capsys.readouterr()

    assert exit_code == 1
    assert "Erro durante a execução do pipeline: Falha de rede simulada" in captured.err