import os
from dotenv import load_dotenv
from pathlib import Path

env_path=Path('.env')
load_dotenv(dotenv_path=env_path)

class Settings:
    PROJECT_NAME: str = "Access_Control"
    PROJECT_VERSION: str = "1.0.0"

    DB_USER: str = os.getenv("DB_USER")
    DB_PASSWORD: str = os.getenv("DB_PASSWORD")
    DB_SERVER: str = os.getenv("DB_HOST", "localhost")
    DB_PORT: int = int(os.getenv("DB_PORT", 5432))
    DB: str = os.getenv("DB_NAME")

    @property
    def DATABASE_URL(self) -> str:
        return (
            f"postgresql://{self.DB_USER}:{self.DB_PASSWORD}"
            f"@{self.DB_SERVER}:{self.DB_PORT}/{self.DB}"
        )

settings= Settings()