from sqlalchemy import Column, String, ForeignKey
from sqlalchemy.orm import relationship
from controller.db.base_class import  Base

class Card(Base):
    __tablename__= "cards"
    
    card_id= Column(String, primary_key= True, index=True)
    status= Column(String, nullable=False)
    user_id= Column(String, ForeignKey("users.user_id"), nullable=False)
    
    user= relationship("User", back_populates="cards")