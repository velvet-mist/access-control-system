from sqlalchemy import Column, ForeignKey, Integer, String
from sqlalchemy.orm import relationship

from controller.db.base_class import Base

class Card(Base):
    __tablename__ = "cards"

    id = Column(Integer, primary_key=True)
    card_id = Column(String, unique=True)
    user_id = Column(ForeignKey("users.id"))

    user = relationship("User", back_populates="cards")
