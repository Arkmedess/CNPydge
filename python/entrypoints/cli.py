"""CLI do CNPydge (Typer).

Comandos:
- importar: baixa e importa a base da RFB
- status: mostra status da importacao
- serve: inicia a API REST

Ver skill: python-orchestration.
Nunca implementar aqui logica de parsing/normalizacao -- delegado ao Rust via PyO3.
"""

import asyncio
from pathlib import Path

import typer
from loguru import logger

app = typer.Typer(help="CNPydge -- Pipeline de ingestao de CNPJ")


@app.command()
def importar(
    config_path: str = typer.Option("config.toml", help="Caminho do arquivo de configuracao"),
    files: list[str] = typer.Option(None, help="Arquivos especificos para importar"),
    batch_size: int = typer.Option(100_000, help="Tamanho do lote"),
    dsn: str = typer.Option(None, help="DSN do PostgreSQL (sobrepoe config)"),
) -> None:
    """Importa a base da Receita Federal.

    Baixa os arquivos CSV da RFB, processa com o pipeline Rust
    e persiste no PostgreSQL via COPY.
    """
    from python.config.settings import get_settings

    settings = get_settings()
    postgres_dsn = dsn or settings.postgres_dsn.get_secret_value()

    logger.info("Iniciando importacao CNPJ...")
    logger.info("PostgreSQL: {}", postgres_dsn.split("@")[-1] if "@" in postgres_dsn else postgres_dsn)
    logger.info("Batch size: {}", batch_size)

    # Step 1: Download files
    download_dir = Path(settings.download_dir)
    download_dir.mkdir(parents=True, exist_ok=True)

    if files:
        filenames = files
    else:
        from python.features.updater.downloader import RFB_FILES
        filenames = RFB_FILES

    logger.info("Baixando {} arquivos...", len(filenames))

    downloaded = asyncio.run(
        _download_files(download_dir, filenames)
    )

    if not downloaded:
        logger.error("Nenhum arquivo baixado. Abortando.")
        raise typer.Exit(1)

    logger.info("{} arquivos baixados com sucesso.", len(downloaded))

    # Step 2: Import via Rust pipeline
    try:
        import cnpj_core
    except ImportError:
        logger.error(
            "Modulo Rust 'cnpj_core' nao encontrado. "
            "Execute 'maturin develop --release' primeiro."
        )
        raise typer.Exit(1)

    csv_files = [str(f) for f in downloaded if f.suffix == ".csv"]
    if not csv_files:
        # If only ZIPs, we need to extract them first
        logger.info("Extraindo arquivos ZIP...")
        csv_files = asyncio.run(_extract_zips(downloaded, download_dir))

    if not csv_files:
        logger.error("Nenhum arquivo CSV encontrado para importar.")
        raise typer.Exit(1)

    logger.info("Importando {} arquivos CSV...", len(csv_files))

    stats = cnpj_core.importar(
        files=csv_files,
        dsn=postgres_dsn,
        batch_size=batch_size,
    )

    logger.info("Importacao concluida!")
    logger.info("  Registros processados: {}", stats.records_processed)
    logger.info("  Bytes lidos: {:.1} MB", stats.bytes_read / 1_048_576)
    logger.info("  Lotes enviados: {}", stats.batches_sent)
    logger.info("  Registros gravados: {}", stats.records_written)
    logger.info("  Erros: {}", stats.errors)
    logger.info("  Tempo: {:.1}s", stats.elapsed_secs)
    logger.info("  Throughput: {:.0} registros/s", stats.records_per_sec)
    logger.info("  Throughput: {:.1} MB/s", stats.mb_per_sec)


@app.command()
def status(
    config_path: str = typer.Option("config.toml", help="Caminho do arquivo de configuracao"),
) -> None:
    """Mostra o status da importacao."""
    from python.config.settings import get_settings

    settings = get_settings()
    download_dir = Path(settings.download_dir)

    if not download_dir.exists():
        logger.info("Diretorio de cache nao existe: {}", download_dir)
        return

    from python.features.updater.downloader import list_cached_files

    cached = list_cached_files(download_dir)
    logger.info("Arquivos em cache: {}", len(cached))
    for f in cached:
        size_mb = f.stat().st_size / 1_048_576
        logger.info("  {} ({:.1} MB)", f.name, size_mb)


async def _download_files(download_dir: Path, filenames: list[str]) -> list[Path]:
    """Download files from RFB."""
    from python.features.updater.downloader import download_all

    return await download_all(download_dir, files=filenames)


async def _extract_zips(zip_files: list[Path], dest_dir: Path) -> list[str]:
    """Extract ZIP files and return CSV paths."""
    import zipfile

    csv_files = []
    for zip_path in zip_files:
        if not zipfile.is_zipfile(zip_path):
            logger.warning("Nao e um ZIP valido: {}", zip_path)
            continue

        logger.info("Extraindo {}...", zip_path.name)
        with zipfile.ZipFile(zip_path, "r") as zf:
            # Security: check for zip bomb
            total_size = sum(info.file_size for info in zf.infolist())
            if total_size > 10 * 1024 * 1024 * 1024:  # 10GB limit
                logger.error("Possivel zip bomb detectado em {}: {} bytes", zip_path.name, total_size)
                continue

            zf.extractall(dest_dir)
            for info in zf.infolist():
                if info.filename.endswith(".csv"):
                    csv_files.append(str(dest_dir / info.filename))

    return csv_files


if __name__ == "__main__":
    app()
