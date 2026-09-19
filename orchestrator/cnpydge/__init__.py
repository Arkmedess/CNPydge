"""Pacote de orquestração do CNPydge com motor nativo em Rust."""

from cnpydge._core import socios_to_parquet, to_parquet, version

__all__: list[str] = ["socios_to_parquet", "to_parquet", "version"]
