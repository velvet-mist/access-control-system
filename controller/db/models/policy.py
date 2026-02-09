from sqlalchemy import Column, Integer, String, Boolean
from controller.db.base_class import Base

class Policy(Base):
    __tablename__ = "policies"

    id = Column(Integer, primary_key=True, index=True)

    role = Column(String, nullable=False)
    machine_id = Column(String, nullable=False)
    command = Column(String, nullable=False)

    allow = Column(Boolean, default=False)
