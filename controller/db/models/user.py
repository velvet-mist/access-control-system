# -- Active: 1770606449641@@127.0.0.1@5432@access_control
from sqlalchemy import UUID, Column, ForeignKey, Integer, String, DateTime, PrimaryKeyConstraint
from sqlalchemy.orm import relationship
from controller.db.base_class import Base
import datetime
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.sql import func
import uuid

class User(Base):
    __tablename__ = "users"
    user_id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    created_at = Column(DateTime(timezone=True), server_default=func.now(), nullable=False)

    roles = relationship("UserRole", back_populates="user")
    cards = relationship(
        "Card",
        back_populates="user",
        cascade="all, delete-orphan"
    )

class Role(Base):
    __tablename__ = "roles"
    role_id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    name = Column(String, nullable=False, unique=True)
    
    users = relationship("UserRole", back_populates="role")

class UserRole(Base):
    __tablename__ = "user_roles"

    user_id = Column(UUID(as_uuid=True), ForeignKey("users.user_id"), nullable=False)
    role_id = Column(UUID(as_uuid=True), ForeignKey("roles.role_id"), nullable=False)

    __table_args__ = (
        PrimaryKeyConstraint('user_id', 'role_id'),
    )

    user = relationship("User", back_populates="roles")
    role = relationship("Role", back_populates="users")
