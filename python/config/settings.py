"""Configuracao (Pydantic). Ver skill: python-orchestration."""
from pydantic import SecretStr
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_prefix="CNPJ_", toml_file="config.toml")

    postgres_dsn: SecretStr
    batch_size: int = 100_000
    download_dir: str = "./data/cache"
