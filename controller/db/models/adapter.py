from sqlalchemy import Column, Integer, String
from controller.db.base_class import Base

class adapters(Base):
    __tablename__= "adapters"
    adapter_id= Column(Integer, primary_key=True, index= True)
    token= Column(String, index=True)
    status= Column(String, nullable= False)