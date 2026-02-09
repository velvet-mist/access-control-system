from sqlalchemy import Column, Integer, String
from controller.db.base_class import Base
from sqlalchemy.orm import relationship
class User(Base):
    __tablename__ = "users"

    id = Column(Integer, primary_key=True)
    user_id = Column(String, unique=True, nullable=False)

    cards = relationship(
        "Card",
        back_populates="user",
        cascade="all, delete-orphan"
    )