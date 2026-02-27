from controller.db.deps import get_db
from controller.db.models.adapter import adapters

for adapter in db.query(adapters).all():
    print(f"Token: {adapter.token}, Status: {adapter.status}")
