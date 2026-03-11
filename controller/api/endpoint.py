from fastapi import APIRouter
from pydantic import BaseModel

router = APIRouter()


class InspectionPayload(BaseModel):
    device_id: str
    result: str
    raw: str


@router.post("/inspection")
async def receive_inspection(payload: InspectionPayload):
    return {"status": "logged", "device_id": payload.device_id}
