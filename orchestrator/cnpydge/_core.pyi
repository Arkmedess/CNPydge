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
    """Retorna a versão semântica da engine nativa compilada em Rust.

    Returns:
        String contendo a versão definida no Cargo.toml do motor nativo.
    """

def to_parquet(src: str, dst: str, kind: TableKindStr | str | None = None) -> int:
    """Converte um arquivo CSV do CNPJ para Parquet em disco liberando o GIL.

    Args:
        src: Caminho do arquivo CSV de origem no disco.
        dst: Caminho de saída do arquivo Parquet gerado.
        kind: Identificador da tabela suportada (ex.: 'empresas', 'socios', etc.).

    Returns:
        Número total de registros convertidos e gravados.

    Raises:
        RuntimeError: Se ocorrer erro de I/O, parsing de registros ou esquema Arrow inválido.
    """
