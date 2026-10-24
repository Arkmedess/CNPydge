"""Submódulo de contratos de API e fachadas de alto nível do CNPydge."""

from cnpydge.api.converter import VALID_KINDS, TableKindStr, run_pipeline, to_parquet

__all__: list[str] = [
    "VALID_KINDS",
    "TableKindStr",
    "run_pipeline",
    "to_parquet",
]