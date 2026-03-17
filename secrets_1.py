
from fastapi import Depends, FastAPI
from fastapi.security import APIKeyCookie

app = FastAPI()

cookie_scheme = APIKeyCookie(name="session")

@app.get("/items/")
async def read_items(session: str = Depends(cookie_scheme)):
    return {"session": session}

 