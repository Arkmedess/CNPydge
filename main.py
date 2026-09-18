"""Ponto de entrada de demonstração e teste do CNPydge."""

import cnpydge


def main() -> None:
    """Executa a verificação inicial do motor nativo."""
    print(f"CNPydge inicializado com sucesso! Versão do motor: {cnpydge.version()}")


if __name__ == "__main__":
    main()
