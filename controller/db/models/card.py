from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy import Column, ForeignKey, String
import uuid
from sqlalchemy.orm import relationship
from controller.db.base_class import Base

class Card(Base):
    __tablename__ = "cards"

    # Using String for card_id since RFID readers output plain strings (not UUIDs)
    card_id = Column(String, primary_key=True)
    user_id = Column(
        UUID(as_uuid=True),
        ForeignKey("users.user_id"),
        nullable=False
    )
    status = Column(String, default="active", nullable=False)

    user = relationship("User", back_populates="cards")
