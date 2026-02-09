from sqlalchemy import Column, String
from sqlalchemy.orm import relationship
from controller.db.base_class import Base

class User(Base):
    __tablename__ = "users"

    user_id = Column(String, primary_key=True)
    display_name = Column(String)
    role = Column(String, nullable=False)

    cards = relationship("Card", back_populates="user")
