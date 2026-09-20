"""Suíte de testes para o orquestrador de conversão do CNPydge."""

from pathlib import Path

import cnpydge
import pytest


def test_to_parquet_missing_source(
    tmp_path: Path, caplog: pytest.LogCaptureFixture
) -> None:
    """Garante emissão de log ERROR e levantamento de FileNotFoundError quando a origem inexiste."""
    missing_file = tmp_path / "inexistente.csv"
    dest_file = tmp_path / "saida.parquet"

    with pytest.raises(FileNotFoundError):
        cnpydge.to_parquet(missing_file, dest_file, kind="empresas")

    assert "não encontrado" in caplog.text


def test_to_parquet_invalid_kind(
    tmp_path: Path, caplog: pytest.LogCaptureFixture
) -> None:
    """Garante emissão de log ERROR e levantamento de ValueError para tabela desconhecida."""
    src = tmp_path / "teste.csv"
    src.write_text("dummy", encoding="utf-8")
    dst = tmp_path / "saida.parquet"

    with pytest.raises(ValueError):
        cnpydge.to_parquet(src, dst, kind="tabela_inexistente")

    assert "não suportada" in caplog.text


def test_to_parquet_success_and_metrics_logging(
    tmp_path: Path, caplog: pytest.LogCaptureFixture
) -> None:
    """Verifica conversão bem-sucedida e emissão de logs INFO com métricas de vazão."""
    src = tmp_path / "empresas.csv"
    dst = tmp_path / "empresas.parquet"
    src.write_text(
        '"12345678";"EMPRESA TESTE";"2062";"49";"1000,00";"01";""\n',
        encoding="utf-8",
    )

    with caplog.at_level("INFO"):
        total = cnpydge.to_parquet(src, dst, kind="empresas")

    assert total == 1
    assert dst.exists()
    assert "linhas processadas" in caplog.text


def test_to_parquet_engine_error_logged(
    tmp_path: Path, caplog: pytest.LogCaptureFixture
) -> None:
    """Valida o registro de log ERROR com causa raiz quando o motor nativo encontra erro de I/O."""
    src = tmp_path / "empresas.csv"
    src.write_text(
        '"12345678";"EMPRESA TESTE";"2062";"49";"1000,00";"01";""\n',
        encoding="utf-8",
    )
    # Passar um diretório existente como dst força erro de criação de arquivo no Rust
    dst_dir = tmp_path / "diretorio_invalido"
    dst_dir.mkdir()

    with pytest.raises(RuntimeError):
        cnpydge.to_parquet(src, dst_dir, kind="empresas")

    assert "Falha no motor nativo" in caplog.text
