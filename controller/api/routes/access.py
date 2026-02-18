from fastapi import APIRouter, Depends, HTTPException
from sqlalchemy.orm import Session
from pydantic import BaseModel
import httpx
import os

from controller.core.access_control import check_access, check_card_exists
from controller.core.security import verify_adapter
from controller.db.deps import get_db
from controller.db.models.card import Card
from controller.db.models.user import User, UserRole, Role

router = APIRouter()

# Rust adapter settings
RUST_ADAPTER_URL = os.getenv("RUST_ADAPTER_URL", "http://localhost:8080")
RUST_ADAPTER_TOKEN = os.getenv("RUST_ADAPTER_TOKEN", "done")

# Pydantic models for request/response
class CheckAccessQueryParams(BaseModel):
    card_id: str
    command: str

class RegisterCardRequest(BaseModel):
    card_id: str
    user_name: str
    role_name: str


@router.get("/check-access")
async def check_access_endpoint(
    card_id: str,
    machine_id: str,
    command: str,
    db: Session = Depends(get_db)
):
    """
    Check if a card has access.
    Returns:
    - "ALLOW" or "DENY" if card exists and is active
    - "NEW_CARD" if card is not registered in the system
    """
    card_exists, status = check_card_exists(db, card_id)
    
    if not card_exists:
        # Card not found - new card
        return {"decision": "NEW_CARD", "message": "Card not registered"}
    
    if status == "inactive":
        return {"decision": "DENY", "message": "Card is inactive"}
    
    # Card exists and is active - check access via policy
    allowed = check_access(
        db=db,
        card_id=card_id,
        command=command,
    )
    
    decision = "ALLOW" if allowed else "DENY"
    return {"decision": decision}


@router.get("/roles")
async def list_roles(
    db: Session = Depends(get_db)
):
    """
    Get list of available roles in the system.
    """
    roles = db.query(Role).all()
    return {
        "roles": [{"name": r.name, "role_id": str(r.role_id)} for r in roles]
    }


@router.post("/register-card")
async def register_card(
    request: RegisterCardRequest,
    adapter=Depends(verify_adapter),
    db: Session = Depends(get_db)
):
    """
    Register a new user with a card.
    This endpoint is called when a new card is detected and the user wants to create an account.
    """
    # Check if card already exists
    card_exists, _ = check_card_exists(db, request.card_id)
    if card_exists:
        raise HTTPException(
            status_code=400,
            detail="Card already registered"
        )
    
    # Check if role exists
    role = db.query(Role).filter(Role.name == request.role_name).first()
    if not role:
        raise HTTPException(
            status_code=400,
            detail=f"Role '{request.role_name}' not found. Available roles: operator, engineer, admin"
        )
    
    # Create user
    user = User()
    db.add(user)
    db.flush()  # Get user_id
    
    # Create card linked to user
    card = Card(
        card_id=request.card_id,
        user_id=user.user_id,
        status="active"
    )
    db.add(card)
    
    # Assign role to user
    user_role = UserRole(
        user_id=user.user_id,
        role_id=role.role_id
    )
    db.add(user_role)
    
    db.commit()
    
    return {
        "status": "success",
        "message": f"User '{request.user_name}' registered successfully with card {request.card_id}",
        "user_id": str(user.user_id),
        "card_id": request.card_id,
        "role": request.role_name
    }


@router.post("/trigger-plc")
async def trigger_plc(
    card_id: str,
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
