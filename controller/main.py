from fastapi import FastAPI
from pathlib import Path
import sys

if __package__ is None or __package__ == "":
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from controller.db.base_class import Base
from controller.db.session import engine
from controller.core.config import settings

import controller.db.models.user
import controller.db.models.card
import controller.db.models.policy
from controller.db.models.audit_log import AuditLog
from controller.api import endpoint
from controller.api.routes import access, audit


def create_tables():
    Base.metadata.create_all(bind=engine)


def start_application():
    app = FastAPI(
        title=settings.PROJECT_NAME,
        version=settings.PROJECT_VERSION,
    )
    create_tables()
    app.include_router(access.router, prefix="/api")
    app.include_router(audit.router, prefix="/api")
    app.include_router(endpoint.router, prefix="/api")
    return app


app = start_application()


@app.get("/")
def home():
    return {"msg": "works?"}
