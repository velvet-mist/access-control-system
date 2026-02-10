from sqlalchemy.dialects.postgresql import UUID
from sqlalchemy import Column, ForeignKey
import uuid
from sqlalchemy.orm import relationship
from controller.db.base_class import Base

class Card(Base):
    __tablename__ = "cards"

    user_id = Column(
        UUID(as_uuid=True),
        ForeignKey("users.user_id"),
        primary_key=True
    )

    user = relationship("User", back_populates="cards")
