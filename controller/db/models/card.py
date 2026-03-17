from sqlalchemy import Column, ForeignKey, String, DateTime
from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy.orm import relationship
from controller.db.base_class import Base


class Card(Base):
    __tablename__ = "cards"

    card_id = Column(String(64), primary_key=True)
    user_id = Column(UUID(as_uuid=True), ForeignKey("users.user_id"), nullable=True)
    status = Column(String, nullable=True)          # 'active', 'revoked', 'expired'
    source_system = Column(String, nullable=True)
    last_synced_at = Column(DateTime, nullable=True)

    user = relationship("User", back_populates="cards", foreign_keys=[user_id])