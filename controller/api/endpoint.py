from fastapi import FastAPI
from pydantic import BaseModel

app = FastAPI()

class InspectionPayLoad(BaseModel):
    device_id: str
    result: str
    raw: str

@app.post("\Inspection")
async def receive_inspection(payload: InspectionPayLoad):
    return { "status": "logged"}