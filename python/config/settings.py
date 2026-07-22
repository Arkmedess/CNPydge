"""Configuracao central do CNPydge (Pydantic).

Carrega de config.toml e variaveis de ambiente com prefixo CNPJ_.
Ver skill: python-orchestration.
"""

from pydantic import SecretStr
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Configuracao do pipeline CNPJ."""

    model_config = SettingsConfigDict(
        env_prefix="CNPJ_",
        toml_file="config.toml",
        env_file=".env",
        env_file_encoding="utf-8",
    )

    # Database
    postgres_dsn: SecretStr = SecretStr("postgresql://localhost:5432/cnpj")

    # Pipeline
    batch_size: int = 100_000
    max_workers: int = 4

    # Download
    download_dir: str = "./data/cache"
    rfb_base_url: str = "https://dadosabertos.rfb.gov.br/CNPJ"

    # Logging
    log_level: str = "INFO"
    log_file: str | None = None

    # API
    api_host: str = "0.0.0.0"
    api_port: int = 8000


# Singleton for convenience
_settings: Settings | None = None


def get_settings() -> Settings:
    """Get or create the global settings instance."""
    global _settings
    if _settings is None:
        _settings = Settings()
    return _settings


def reset_settings() -> None:
    """Reset the global settings (for testing)."""
    global _settings
    _settings = None
