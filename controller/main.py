from fastapi import FastAPI
from controller.core.config import settings
from controller.db.session import engine
from controller.db.base_class import Base

import controller.db.models.user
import controller.db.models.card
import controller.db.models.policy
from controller.db.models.audit_log import AuditLog
from controller.api.routes import audit
from controller.api.routes import access
from adapter import adapter
def create_tables():
    Base.metadata.create_all(bind=engine)
    
def start_application():
    app= FastAPI(title=settings.PROJECT_NAME, version=settings.PROJECT_VERSION)
    create_tables()
    return app
app=start_application()
app.include_router(access.router, prefix="/api")
app.include_router(audit.router, prefix="/api")
@app.get("/")

def home():
    return {"msg":"works?"}