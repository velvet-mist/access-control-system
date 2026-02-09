from fastapi import Header, HTTPException, Depends
from sqlalchemy.orm import Session
from controller.db.deps import get_db
from controller.db.models.adapter import adapters

def verify_adapter(
    x_adapter_token:str= Header(...),
    db:Session=Depends(get_db)
)-> adapters:
    adapter=db.query(adapters).filter(
        adapters.token==x_adapter_token,
        adapters.status=="active" 
    ).first()
    if not adapter:
        raise HTTPException(status_code=401, detail="Invalid adapter")

    return adapter