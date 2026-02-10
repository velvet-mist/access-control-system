from sqlalchemy import Column, Integer, String, ForeignKey
from sqlalchemy.orm import relationship
from controller.db.base_class import Base

class Card(Base):
    __tablename__ = "cards"

    id = Column(Integer, primary_key=True)
    user_id = Column(Integer, ForeignKey("users.id"), nullable=False)

    user = relationship("User", back_populates="cards")
