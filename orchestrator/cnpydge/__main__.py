"""Ponto de entrada canônico para execução do CNPydge via módulo Python ou contêiner."""

from __future__ import annotations

import argparse
import sys

import cnpydge


def build_parser() -> argparse.ArgumentParser:
    """Constrói o parser de argumentos de linha de comando para o despachador principal."""
    parser = argparse.ArgumentParser(
        prog="python -m cnpydge",
        description="CNPydge - Pipeline de alta vazão para Dados Públicos de CNPJ da Receita Federal.",
        epilog="Execute 'python -m cnpydge <comando> --help' para detalhes sobre subcomandos específicos.",
    )
    parser.add_argument(
        "-v",
        "--version",
        action="store_true",
        help="Exibe a versão atual do sistema e encerra.",
    )

    subparsers = parser.add_subparsers(
        dest="command",
        title="Comandos disponíveis",
        description="Subcomandos operacionais suportados",
    )

    run_parser = subparsers.add_parser(
        "run",
        help="Executa a esteira completa de download, extração atômica e conversão para Parquet.",
    )
    run_parser.add_argument(
        "--output-dir",
        "-o",
        required=True,
        help="Diretório de destino para armazenamento dos arquivos Parquet gerados.",
    )
    run_parser.add_argument(
        "--tables",
        "-t",
        default=None,
        help="Lista de tabelas separadas por vírgula a processar (ex.: 'empresas,socios'). Se omitido, processa todas.",
    )
    run_parser.add_argument(
        "--profile",
        "-p",
        default="balanced_rolling_eviction",
        choices=[
            "balanced_rolling_eviction",
            "performance_batch_retain",
            "archival_permanent_zip",
        ],
        help="Perfil declarativo de retenção de armazenamento em disco temporário.",
    )

    return parser


def main(argv: list[str] | None = None) -> int:
    """Função principal executável.

    Args:
        argv: Lista opcional de argumentos de linha de comando (default: sys.argv[1:]).

    Returns:
        Código de saída do processo (0 para sucesso, 1 ou 2 para falhas).
    """
    if argv is None:
        argv = sys.argv[1:]

    parser = build_parser()

    if not argv:
        parser.print_help()
        return 0

    args = parser.parse_args(argv)

    if args.version:
        sys.stdout.write(f"CNPydge v{cnpydge.version()}\n")
        return 0

    if args.command == "run":
        tables = [t.strip() for t in args.tables.split(",") if t.strip()] if args.tables else None
        try:
            cnpydge.run_pipeline(
                output_dir=args.output_dir,
                tables=tables,
                profile=args.profile,
            )
            return 0
        except (RuntimeError, ValueError, OSError, TimeoutError) as exc:
            sys.stderr.write(f"Erro durante a execução do pipeline: {exc}\n")
            return 1

    parser.print_help()
    return 0


if __name__ == "__main__":
    sys.exit(main())