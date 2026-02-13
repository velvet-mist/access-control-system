from sqlalchemy import Column, Integer, String, DateTime
from sqlalchemy.sql import func
from controller.db.base_class import Base

class AuditLog(Base):
    __tablename__="audit_logs"
    id= Column(Integer, primary_key=True, index=True)
    
    adapter_id= Column(String, index=True)
    
    card_id= Column(String, index=True)
    user_id= Column(String, index= True)
    
    command= Column(String, nullable=False)
    
    decision= Column(String, nullable=False)
    reason= Column(String, nullable=True)
    
    created_at= Column(DateTime(timezone=True), server_default=func.now())
    
    