from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
import httpx
import os

from controller.core.access_control import check_access
from controller.core.security import verify_adapter
from controller.db.deps import get_db

router= APIRouter()

# Rust adapter settings
RUST_ADAPTER_URL = os.getenv("RUST_ADAPTER_URL", "http://localhost:8080")
RUST_ADAPTER_TOKEN = os.getenv("RUST_ADAPTER_TOKEN", "done")

@router.post("/trigger-plc")
async def trigger_plc(
    card_id: str,
    machine_id: str,
    command: str,
    adapter= Depends(verify_adapter),
    db: Session= Depends(get_db)
):
    """
    Trigger the Rust adapter to control the Keyence PLC.
    This endpoint is called after access is checked to signal the PLC.
    """
    # First check access
    allowed = check_access(
        db=db,
        card_id=card_id,
        command=command,
        machine_id=machine_id,
        adapter_id=adapter.adapter_id
    )
    
    decision = "ALLOW" if allowed else "DENY"
    
    # Call Rust adapter to trigger PLC
    try:
        async with httpx.AsyncClient(timeout=10.0) as client:
            response = await client.post(
                f"{RUST_ADAPTER_URL}/api/trigger",
                json={
                    "card_id": card_id,
                    "machine_id": machine_id,
                    "command": command,
                    "decision": decision
                },
                headers={"Authorization": f"Bearer {RUST_ADAPTER_TOKEN}"}
            )
            
            if response.status_code != 200:
                raise HTTPException(
                    status_code=response.status_code,
                    detail=f"Rust adapter error: {response.text}"
                )
            
            plc_response = response.json()
            
            return {
                "decision": decision,
                "plc_triggered": True,
                "plc_register": plc_response.get("plc_register"),
                "plc_value": plc_response.get("value")
            }
            
    except httpx.RequestError as e:
        raise HTTPException(
            status_code=503,
            detail=f"Failed to connect to Rust adapter: {str(e)}"
        )


# @router.callbacks("/override token access")
