from fastapi import FastAPI, Depends, HTTPException
from fastapi.security import HTTPBearer
import jwt

app = FastAPI()
security = HTTPBearer()

roles = {"operator": ["read"], "engineer": ["read", "write", "trigger"]}

async def get_current_role(token: str = Depends(security)):
    try:
        payload = jwt.decode(token.credentials, "secret", algorithms=["HS256"])
        return payload["role"]
    except:
        raise HTTPException(401, "Invalid token")

@app.post("/trigger_inspection")
async def trigger(role: str = Depends(get_current_role)):
    if "trigger" not in roles.get(role, []):
        raise HTTPException(403, "Insufficient role")
    # Your API: modbus_client.write_register(100, 1)  # Trigger DM100
    return {"status": "OK"}

@app.get("/get_results")
async def results(role: str = Depends(get_current_role)):
    if "read" not in roles.get(role, []): raise HTTPException(403)
    # Your API read
    return {"ng_ok": True}
