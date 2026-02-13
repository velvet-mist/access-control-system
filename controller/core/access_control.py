from sqlalchemy.orm import Session
from controller.db.models.card import Card
from controller.db.models.policy import Policy
from controller.core.audit import log_access

def check_access(
    db: Session,
    card_id: str,
    command: str,
    adapter_id: str
) -> bool:

    card = db.query(Card).filter(
        Card .card_id == card_id,
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

    policy = db.query(Policy).filter(
        Policy.role == card.user.role,
        Policy.command == command,
        Policy.allow == True
    ).first()

    if policy:
        log_access(
            db=db,
            adapter_id=adapter_id,
            card_id=card_id,
            user_id=card.user.user_id,
            command=command,
            decision="ALLOW",
            reason="Policy matched"
        )
        return True

    log_access(
        db=db,
        adapter_id=adapter_id,
        card_id=card_id,
        user_id=card.user.user_id,
        command=command,
        decision="DENY",
        reason="No matching policy"
    )
    return False
