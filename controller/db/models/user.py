from sqlalchemy import Column, ForeignKey, Integer, String, DateTime, PrimaryKeyConstraint
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import relationship
from controller.db.base_class import Base
import uuid


class User(Base):
    __tablename__ = "users"

    user_id = Column(UUID(as_uuid=True), primary_key=True, default=uuid.uuid4)
    display_name = Column(String, nullable=True)
    user_type = Column(String, nullable=True)   # 'employee', 'vendor', 'admin'
    status = Column(String, nullable=True)      # 'active', 'disabled'
    created_at = Column(DateTime, nullable=True)
    card_id = Column(String(255), ForeignKey("cards.card_id"), nullable=True)

    roles = relationship("UserRole", back_populates="user")
    cards = relationship(
        "Card",
        back_populates="user",
        foreign_keys="Card.user_id",
        cascade="all, delete-orphan"
    )


class Role(Base):
    __tablename__ = "roles"

    role_id = Column(Integer, primary_key=True, autoincrement=True)
    name = Column('role_name', String, nullable=False, unique=True)
    description = Column(String, nullable=True)

    users = relationship("UserRole", back_populates="role")


class UserRole(Base):
    __tablename__ = "user_roles"

    user_id = Column(UUID(as_uuid=True), ForeignKey("users.user_id"), nullable=False)
    role_id = Column(Integer, ForeignKey("roles.role_id"), nullable=False)

    __table_args__ = (
        PrimaryKeyConstraint('user_id', 'role_id'),
    )

    user = relationship("User", back_populates="roles")
    role = relationship("Role", back_populates="users")