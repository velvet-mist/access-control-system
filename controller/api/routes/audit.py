from fastapi import APIRouter, Depends
from sqlalchemy.orm import Session
from controller.db.deps import get_db
from controller.db.models.audit_log import AuditLog
from io import StringIO
import csv
from requests import Response

router=APIRouter()

@router.get("/audit-logs")
def get_audit_logs(
    limit: int=50,
    db: Session = Depends(get_db)
):
    logs=(
        db.query(AuditLog)
        .order_by(AuditLog.created_at.desc())
        .limit(limit).all()
    )
    return logs

@router.get("/audit-logs/filter")
def filter_logs(
    decision:str | None= None,
    adapter_id: str | None= None,
    db: Session= Depends(get_db)
):
    q= db.query(AuditLog)
    if decision:
        q= q.filter(AuditLog.decision==decision)
    if adapter_id:
        q= q.filter(AuditLog.adapter_id==adapter_id)
    return q.order_by(AuditLog.created_at.desc()).all()

@router.get("/audit-logs/export")
def export_audit_logs(db: Session= Depends(get_db)):
    logs= db.query(AuditLog).order_by(AuditLog.created_at.desc()).all()
    
    output= StringIO()
    writer= csv.writer(output)
    
    writer.writerow(
        [
            "timestamp","adapter_id","card_id",
            "machine_id","command","decision","reason"
        ]
    )
    for l in logs:
        writer.writerow([
            l.created_at, l.adapter_id, l.card_id,
            l.machine_id, l.command, l.decision,l.reason
        ])
    return Response(
        content= output.getValue(),
        media_type= "text/csv",
        headers={"Content-Disposition": "attachment; filename=audit_logs.csv"}
    )