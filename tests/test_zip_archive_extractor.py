"""Testes unitários para o extrator atômico de arquivos ZIP da RFB."""

import zipfile
from pathlib import Path

import pytest
from cnpydge.pipeline.zip_archive_extractor import StandardZipArchiveExtractor


def test_extract_valid_zip_archive(tmp_path: Path) -> None:
    """Garante que um arquivo ZIP válido da RFB tenha seu CSV extraído com sucesso."""
    zip_path = tmp_path / "Empresas0.zip"
    dest_dir = tmp_path / "extracted"

    # Cria um arquivo zip sintético simulando a RFB
    with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        zf.writestr("K3241.K03200Y0.D40810.EMPRECSV", '"12345678";"EMPRESA TESTE";"2062"\n')

    extractor = StandardZipArchiveExtractor()
    extracted_csv = extractor.extract_csv_from_zip(zip_path, dest_dir)

    assert extracted_csv.is_file()
    assert extracted_csv.suffix == ".csv"
    assert "EMPRESA TESTE" in extracted_csv.read_text(encoding="utf-8")


def test_extract_empty_zip_raises_error(tmp_path: Path) -> None:
    """Verifica se um arquivo ZIP sem arquivos internos lança ValueError explicativo."""
    empty_zip = tmp_path / "empty.zip"
    with zipfile.ZipFile(empty_zip, "w"):
        pass

    extractor = StandardZipArchiveExtractor()
    with pytest.raises(ValueError, match="vazio"):
        extractor.extract_csv_from_zip(empty_zip, tmp_path / "extracted")


def test_extract_missing_zip_raises_file_not_found(tmp_path: Path) -> None:
    """Garante que tentar extrair um arquivo inexistente lance FileNotFoundError."""
    missing_zip = tmp_path / "nao_existe.zip"
    extractor = StandardZipArchiveExtractor()
    with pytest.raises(FileNotFoundError):
        extractor.extract_csv_from_zip(missing_zip, tmp_path / "extracted")
