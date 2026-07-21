"""Entrada da CLI do CNPydge."""
import typer

app = typer.Typer(help="CNPydge")


@app.command()
def importar(config_path: str = "config.toml") -> None:
    """Importa a base da Receita Federal."""
    raise NotImplementedError
