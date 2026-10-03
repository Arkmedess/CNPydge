"""Repositório de persistência de checkpoints em manifesto JSON."""

import json
from pathlib import Path
from typing import Any

from cnpydge.logging import get_logger
from cnpydge.pipeline.partition_models import (
    PartitionProcessingRecord,
    RfbPartitionStage,
)

LOGGER = get_logger("cnpydge.pipeline.checkpoint")


class JsonCheckpointManifestStore:
    """Implementa a persistência de checkpoints do pipeline em arquivo JSON."""

    def __init__(self, manifest_path: str | Path) -> None:
        """Inicializa o repositório de manifesto garantindo o carregamento do arquivo.

        Args:
            manifest_path: Caminho do arquivo JSON no sistema de arquivos.
        """
        self._path: Path = Path(manifest_path)
        self._state: dict[str, dict[str, Any]] = self._load()

    def _load(self) -> dict[str, dict[str, Any]]:
        """Carrega os dados persistidos do manifesto ou retorna dicionário vazio."""
        if self._path.is_file():
            try:
                return json.loads(self._path.read_text(encoding="utf-8"))
            except (json.JSONDecodeError, OSError) as err:
                LOGGER.warning(
                    "Manifesto de checkpoint corrompido ou inacessível em '%s': %s. Reiniciando estado em memória.",
                    self._path,
                    err,
                )
                return {}
        return {}

    def _persist(self) -> None:
        """Grava o estado atual de forma atômica utilizando um arquivo temporário."""
        self._path.parent.mkdir(parents=True, exist_ok=True)
        LOGGER.debug(
            "Persistindo manifesto de checkpoint com %d partições em '%s'.",
            len(self._state),
            self._path,
        )
        tmp_file = self._path.with_suffix(".tmp")
        tmp_file.write_text(json.dumps(self._state, indent=2, ensure_ascii=False), encoding="utf-8")
        tmp_file.replace(self._path)

    def get_partition_stage(self, partition_filename: str) -> RfbPartitionStage:
        """Retorna o estágio atual da partição ou PENDING_DOWNLOAD por padrão."""
        entry = self._state.get(partition_filename)
        if not entry:
            return RfbPartitionStage.PENDING_DOWNLOAD
        return RfbPartitionStage(entry["stage"])

    def record_partition_progress(self, record: PartitionProcessingRecord) -> None:
        """Grava os dados de auditoria da partição e sincroniza o manifesto no disco."""
        self._state[record.partition.remote_filename] = {
            "table": record.partition.table_canonical_name,
            "stage": str(record.stage),
            "parquet_path": str(record.parquet_output_path) if record.parquet_output_path else None,
            "rows_converted": record.rows_converted,
            "error_message": record.error_message,
        }
        self._persist()

    def is_partition_completed(self, partition_filename: str) -> bool:
        """Verifica se a partição já foi convertida para Parquet com sucesso."""
        return self.get_partition_stage(partition_filename) == RfbPartitionStage.CONVERSION_COMPLETED
