"""Suíte de testes rigorosa para o orquestrador de conversão do CNPydge."""

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

    with pytest.raises(ValueError, match="não suportada"):
        cnpydge.to_parquet(src, dst, kind="tabela_inexistente")

    assert "não suportada" in caplog.text


@pytest.mark.parametrize(
    ("kind", "fixture_name", "expected_rows"),
    [
        ("empresas", "sample_empresa_csv_content", 2),
        ("socios", "sample_socio_csv_content", 1),
        ("estabelecimentos", "sample_estabelecimento_csv_content", 1),
        ("simples", "sample_simples_csv_content", 1),
        ("cnaes", "sample_cnae_csv_content", 1),
        ("municipios", "sample_municipio_csv_content", 1),
    ],
)
def test_to_parquet_all_entity_kinds_integrity(
    kind: str,
    fixture_name: str,
    expected_rows: int,
    request: pytest.FixtureRequest,
    tmp_path: Path,
) -> None:
    """Garante a conversão e integridade física de Parquet para todas as entidades suportadas.

    Prevenção de bug: Valida que cada entidade cadastral é processada pelo motor nativo
    e produz um arquivo Parquet com o cabeçalho e rodapé mágicos ('PAR1') do padrão Apache.
    """
    csv_content: str = request.getfixturevalue(fixture_name)
    src = tmp_path / f"{kind}.csv"
    dst = tmp_path / f"{kind}.parquet"
    src.write_text(csv_content, encoding="utf-8")

    total_rows = cnpydge.to_parquet(src, dst, kind=kind)

    assert total_rows == expected_rows
    assert dst.is_file()
    assert dst.stat().st_size > 12  # Mínimo para conter cabeçalho, metadata e rodapé PAR1

    content_bytes = dst.read_bytes()
    # Padrão Apache Parquet: 4 bytes iniciais e finais são 'PAR1'
    assert content_bytes[:4] == b"PAR1", f"Cabeçalho mágico Parquet inválido para {kind}"
    assert content_bytes[-4:] == b"PAR1", f"Rodapé mágico Parquet inválido para {kind}"


def test_to_parquet_empty_file_edge_case(tmp_path: Path) -> None:
    """Garante que a conversão de um arquivo vazio retorne 0 linhas sem pânico no Rust."""
    src = tmp_path / "vazio.csv"
    dst = tmp_path / "vazio.parquet"
    src.write_text("", encoding="utf-8")

    total_rows = cnpydge.to_parquet(src, dst, kind="empresas")

    assert total_rows == 0
    assert not dst.exists()


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
