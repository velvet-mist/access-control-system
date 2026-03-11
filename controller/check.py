from controller.db.models.adapter import adapters
from controller.db.session import SessionLocal


def main() -> None:
    db = SessionLocal()
    try:
        for adapter in db.query(adapters).all():
            print(f"Token: {adapter.token}, Status: {adapter.status}")
    finally:
        db.close()


if __name__ == "__main__":
    main()
