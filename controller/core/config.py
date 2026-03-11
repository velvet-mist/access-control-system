import os
from pathlib import Path
from dotenv import load_dotenv

env_path = Path(__file__).resolve().parents[2] / ".env"
load_dotenv(dotenv_path=env_path)


def _get_env(name: str, default: str | None = None) -> str | None:
    value = os.getenv(name, default)
    if value is None:
        return None
    return value.strip()


class Settings:
    PROJECT_NAME: str = "Access_Control"
    PROJECT_VERSION: str = "1.0.0"

    DB_USER: str | None = _get_env("DB_USER")
    DB_PASSWORD: str | None = _get_env("DB_PASSWORD")
    DB_SERVER: str = _get_env("DB_HOST", "localhost") or "localhost"
    DB_PORT: int = int(_get_env("DB_PORT", "5432") or "5432")
    DB: str | None = _get_env("DB_NAME")

    @property
    def DATABASE_URL(self) -> str:
        return (
            f"postgresql://{self.DB_USER}:{self.DB_PASSWORD}"
            f"@{self.DB_SERVER}:{self.DB_PORT}/{self.DB}"
        )


settings = Settings()
