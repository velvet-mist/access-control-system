# Setup & Deployment

## Prerequisites
- Docker & docker-compose
- PostgreSQL (or Docker)
- Rust toolchain (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)

## Quickstart (Docker)
```bash
docker compose up -d  # Starts backend, rust-adapter, postgres
```

## Manual/Local Dev
1. **DB**:
   ```bash
   cd controller
   pip install -r requirements.txt
   # Run migrations (if alembic): alembic upgrade head
   python -m controller.db.base  # Or use pg_dump from access.session.sql
   ```

2. **Python Backend**:
   ```bash
   cd controller
   export DATABASE_URL=postgresql://user:pass@localhost/ac_db
   uvicorn main:app --host 0.0.0.0 --port 8000 --reload
   ```
   - API Docs: http://localhost:8000/docs (Swagger)

3. **Rust Adapter**:
   ```bash
   cd rust-adapter
   cp .env.example .env  # Set PLC_IP, etc.
   cargo run
   ```

## Hardware Setup
See [hardware.md](./hardware.md).

## Env Vars
- `DATABASE_URL`: Postgres conn
- Rust: `PLC_HOST=192.168.1.100:502`, `BACKEND_URL=http://localhost:8000`

Test: Tap card → POST /access → PLC command.

