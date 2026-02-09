from fastapi import APIRouter, Depends
from sqlalchemy.orm import Session

from controller.core.access_control import check_access
from controller.core.security import verify_adapter
from controller.db.deps import get_db

router= APIRouter()

@router.post("/check-access")
def check_access_api(
    card_id:str,
    machine_id:str,
    command:str,
    adapter= Depends(verify_adapter),
    db: Session= Depends(get_db)
):
    allowed= check_access(
        db=db,
        card_id=card_id,
        command=command,
        machine_id=machine_id,
        adapter_id= adapter.adapter_id
    )
    
    return {"decision":"ALLOW" if allowed else "DENY"}