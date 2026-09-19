"""Pacote de orquestração do CNPydge com motor nativo em Rust."""

from cnpydge._core import to_parquet, version

__all__: list[str] = ["to_parquet", "version"]
