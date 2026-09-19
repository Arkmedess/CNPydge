from typing import Literal

type TableKindStr = Literal[
    "empresas",
    "socios",
    "estabelecimentos",
    "simples",
    "cnaes",
    "motivos",
    "municipios",
    "naturezas",
    "paises",
    "qualificacoes",
]

def version() -> str:
    """Retorna a versão da engine compilada em Rust."""

def to_parquet(src: str, dst: str, kind: TableKindStr | str | None = None) -> int:
    """Converte um arquivo CSV do CNPJ para Parquet em disco liberando o GIL."""
