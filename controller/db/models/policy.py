from sqlalchemy import Column, ForeignKey, Integer, String, Boolean, Time
from sqlalchemy.orm import relationship
from controller.db.base_class import Base


class Policy(Base):
    __tablename__ = "policies"

    policy_id = Column('policy_id', Integer, primary_key=True, autoincrement=True)
    role_id = Column(Integer, ForeignKey("roles.role_id"), nullable=True)
    machine_id = Column(String, ForeignKey("machines.machine_id"), nullable=True)
    command = Column(String, nullable=True)
    allow = Column(Boolean, default=False)
    time_start = Column(Time, nullable=True)
    time_end = Column(Time, nullable=True)

    role = relationship("Role", backref="policies")