from controller.db.models.audit_log import AuditLog
from sqlalchemy.orm import Session

def log_access(
    db: Session,
    adapter_id: str,
    card_id: str,
    user_id: str | None,
    command: str,
    decision:str,
    reason:str | None= None
):
    log= AuditLog(
        adapter_id= adapter_id,
        card_id= card_id,
        user_id=user_id,
        command= command,
        decision=decision,
        reason=reason
    )
    
    db.add(log)
    db.commit()