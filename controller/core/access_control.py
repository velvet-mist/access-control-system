# -- Active: 1770606449641@@127.0.0.1@5432@access_control
from sqlalchemy.orm import Session
from controller.db.models.card import Card
from controller.db.models.policy import Policy
from controller.db.models.user import User, UserRole, Role
from controller.core.audit import log_access

def check_access(
    db: Session,
    card_id: str,
    command: str,
    adapter_id: str
) -> bool:

    card = db.query(Card).filter(
        Card.card_id == card_id,
        Card.status == "active"
    ).first()

    if not card:
        log_access(
            db=db,
            adapter_id=adapter_id,
            card_id=card_id,
            user_id=None,
            command=command,
            decision="DENY",
            reason="Card invalid or inactive"
        )
        return False

    # Get user's role_id through user_roles table
    user_role = db.query(UserRole).filter(
        UserRole.user_id == card.user_id
    ).first()
    
    if not user_role:
        log_access(
            db=db,
            adapter_id=adapter_id,
            card_id=card_id,
            user_id=card.user_id,
            command=command,
            decision="DENY",
            reason="User has no role assigned"
        )
        return False

    # Query policy by role_id
    policy = db.query(Policy).filter(
        Policy.role_id == user_role.role_id,
        Policy.command == command,
        Policy.allow == True
    ).first()

    if policy:
        log_access(
            db=db,
            adapter_id=adapter_id,
            card_id=card_id,
            user_id=card.user_id,
            command=command,
            decision="ALLOW",
            reason="Policy matched"
        )
        return True

    log_access(
        db=db,
        adapter_id=adapter_id,
        card_id=card_id,
        user_id=card.user_id,
        command=command,
        decision="DENY",
        reason="No matching policy"
    )
    return False
