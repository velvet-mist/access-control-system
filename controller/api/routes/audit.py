from fastapi import APIRouter, Depends, HTTPException
from fastapi.responses import Response
from sqlalchemy.orm import Session
from controller.db.deps import get_db
from controller.db.models.audit_log import AuditLog
from controller.db.models.user import User
from controller.db.models.card import Card
from io import StringIO
import csv
from pydantic import BaseModel
from datetime import datetime
from typing import Optional

router = APIRouter()

class UserCreate(BaseModel):
    user_id: str
    card_id: str
    display_name: str
    user_type: str
    status: str
    created_at: datetime


class UserUpdate(BaseModel):
    card_id: Optional[str] = None
    display_name: Optional[str] = None
    user_type: Optional[str] = None
    status: Optional[str] = None

    class Config:
        extra = "forbid"


@router.get("/audit-logs")
def get_audit_logs(
    limit: int = 50,
    db: Session = Depends(get_db)
):
    return (
        db.query(AuditLog)
        .order_by(AuditLog.created_at.desc())
        .limit(limit)
        .all()
    )


@router.get("/audit-logs/filter")
def filter_logs(
    decision: Optional[str] = None,
    adapter_id: Optional[str] = None,
    db: Session = Depends(get_db)
):
    q = db.query(AuditLog)

    if decision:
        q = q.filter(AuditLog.decision == decision)
    if adapter_id:
        q = q.filter(AuditLog.adapter_id == adapter_id)

    return q.order_by(AuditLog.created_at.desc()).all()

@router.get("/audit-logs/export")
def export_audit_logs(db: Session = Depends(get_db)):
    logs = db.query(AuditLog).order_by(AuditLog.created_at.desc()).all()

    output = StringIO()
    writer = csv.writer(output)

    writer.writerow([
        "timestamp",
        "adapter_id",
        "card_id",
        "machine_id",
        "command",
        "decision",
        "reason"
    ])

    for log in logs:
        writer.writerow([
            log.created_at,
            log.adapter_id,
            log.card_id,
            log.machine_id,
            log.command,
            log.decision,
            log.reason
        ])

    return Response(
        content=output.getvalue(),
        media_type="text/csv",
        headers={"Content-Disposition": "attachment; filename=audit_logs.csv"}
    )
@router.post("/user")
def create_user(
    db: Session = Depends(get_db)
):
    user = User()
    db.add(user)
    db.commit()
    db.refresh(user)
    return user

@router.put("/user/{user_id}")
def update_user(
    user_id: str,
    payload: UserUpdate,
    db: Session = Depends(get_db)
):
    user = db.query(User).filter(User.user_id == user_id).first()
    if not user:
        raise HTTPException(status_code=404, detail="User not found")

    update_data = payload.model_dump(exclude_unset=True)

    for field, value in update_data.items():
        setattr(user, field, value)   # value CAN be None → sets NULL

    db.commit()
    db.refresh(user)
    return user

@router.delete("/user/{user_id}")
def delete_user(
    user_id: str,
    db: Session = Depends(get_db)
):
    user = db.query(User).filter(User.user_id == user_id).first()
    if not user:
        raise HTTPException(status_code=404, detail="User not found")

    db.delete(user)
    db.commit()
    return {"detail": "User deleted successfully"}
