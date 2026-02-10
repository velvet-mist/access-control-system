from sqlalchemy import UUID, Column, ForeignKey, Integer, String, DateTime
from sqlalchemy.orm import relationship
from controller.db.base_class import Base
import datetime
from sqlalchemy.dialects.postgresql import UUID
import uuid

class User(Base):
    __tablename__ = "users"
    id = Column(Integer, primary_key=True)
    user_id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    created_at = Column(DateTime, nullable=False)

    roles = relationship("UserRole", back_populates="user")
    cards = relationship(
        "Card",
        back_populates="user",
        cascade="all, delete-orphan"
    )
    
class UserRole(Base):
    __tablename__ = "user_roles"

    id = Column(Integer, primary_key=True)
    user_id = Column(UUID(as_uuid=True), ForeignKey("users.id"), nullable=False)
    role_id = Column(Integer, ForeignKey("roles.id"), nullable=False)

    user = relationship("User", back_populates="roles")
