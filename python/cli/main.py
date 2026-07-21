"""CLI (Typer). Ver skill: python-orchestration.

Comandos previstos: importar, atualizar, bench, serve.
Nunca implementar aqui logica de parsing/normalizacao de registros -- isso e
delegado ao modulo Rust `cnpj_core` via PyO3 (ver skill: pyo3-bridge).
"""
import typer

app = typer.Typer(help="CNPJ Ultra Importer")


@app.command()
def importar(config_path: str = "config.toml") -> None:
    """Importa a base da Receita Federal. TODO: Fase 1 do roadmap."""
    raise NotImplementedError


if __name__ == "__main__":
    app()
