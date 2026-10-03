"""Extrator atômico para descompressão de arquivos ZIP da Receita Federal."""

import shutil
import zipfile
from pathlib import Path

from cnpydge.logging import get_logger

LOGGER = get_logger("cnpydge.pipeline.extractor")


class StandardZipArchiveExtractor:
    """Extrai arquivos compactados da RFB com garantia de atomicidade no sistema de arquivos."""

    def extract_csv_from_zip(
        self,
        zip_archive_path: Path,
        destination_directory: Path,
    ) -> Path:
        """Extrai o conteúdo de um arquivo ZIP da RFB salvando-o com extensão .csv.

        Args:
            zip_archive_path: Caminho do arquivo .zip a ser descompactado.
            destination_directory: Diretório onde o arquivo .csv final será posicionado.

        Returns:
            Caminho absoluto do arquivo CSV extraído com sucesso.

        Raises:
            FileNotFoundError: Se o arquivo zip não existir.
            ValueError: Se o arquivo zip estiver vazio.
        """
        zip_path = Path(zip_archive_path)
        dest_dir = Path(destination_directory)

        if not zip_path.is_file():
            LOGGER.error("Arquivo ZIP não encontrado para extração: '%s'", zip_path)
            raise FileNotFoundError(f"Arquivo ZIP não encontrado: '{zip_path}'")

        dest_dir.mkdir(parents=True, exist_ok=True)
        target_csv_path = dest_dir / f"{zip_path.stem}.csv"
        temp_csv_path = dest_dir / f"{zip_path.stem}.csv.tmp"

        with zipfile.ZipFile(zip_path, "r") as zf:
            members = zf.namelist()
            if not members:
                LOGGER.error("Arquivo ZIP corrompido ou vazio: '%s'", zip_path)
                raise ValueError(f"O arquivo ZIP '{zip_path.name}' está vazio.")

            # Os arquivos da RFB possuem exatamente um arquivo tabular interno
            internal_file_name = members[0]
            LOGGER.info("Extraindo membro '%s' de '%s' -> '%s'", internal_file_name, zip_path.name, target_csv_path.name)

            with zf.open(internal_file_name) as source, temp_csv_path.open("wb") as target:
                shutil.copyfileobj(source, target, length=1024 * 1024)

        temp_csv_path.replace(target_csv_path)
        LOGGER.info("Extração atômica concluída com sucesso: '%s' (%d bytes)", target_csv_path.name, target_csv_path.stat().st_size)
        return target_csv_path
