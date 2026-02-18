from fastapi import FastAPI, Depends, HTTPException
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
import jwt
import os

SECRET_KEY = os.getenv("JWT_SECRET", "dev_secret")
ALGORITHM = "HS256"

ROLE_PERMISSIONS = {
    "operator": {"read"},
    "engineer": {"read", "write", "trigger"},
}

security = HTTPBearer()

def adapter():
    app = FastAPI()

    async def get_current_role(
        credentials: HTTPAuthorizationCredentials = Depends(security),
    ):
        try:
            payload = jwt.decode(
                credentials.credentials,
                SECRET_KEY,
                algorithms=[ALGORITHM],
            )
            return payload.get("role")
        except jwt.InvalidTokenError:
            raise HTTPException(status_code=401, detail="Invalid token")

    def require_permission(permission: str):
        async def checker(role: str = Depends(get_current_role)):
            if permission not in ROLE_PERMISSIONS.get(role, set()):
                raise HTTPException(status_code=403, detail="Insufficient role")
            return role
        return checker

    @app.post("/trigger_inspection")
    async def trigger(role: str = Depends(require_permission("trigger"))):
        # modbus_client.write_register(100, 1)
        return {"status": "OK"}

    @app.get("/get_results")
    async def results(role: str = Depends(require_permission("read"))):
        return {"ng_ok": True}

    return app
