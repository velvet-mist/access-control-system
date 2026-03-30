# Access Control System 🚀

[![Docker](https://img.shields.io/badge/Docker-Compose-blue)](https://docs.docker.com/compose/)
[![Python](https://img.shields.io/badge/Python-FastAPI-yellow)](https://fastapi.tiangolo.com/)
[![Rust](https://img.shields.io/badge/Rust-Tokio-orange)](https://tokio.rs/)

## 🎯 Overview
Card-based RBAC for Keyence CV-X480F machinery. RFID → Rust Adapter (Pi) → Python Policy → PLC.

**Architecture**:
```
RFID → RPi (Rust:8080) → FastAPI:8000 → Postgres → Audit
                    ↓ Modbus
              Keyence CV-X480F
```

Full docs: [/docs](./docs/)

## 📋 Quickstart
```bash
# Clone & run
docker compose up -d  # Backend + Adapter + DB (no clone needed here)

# API
curl -X POST http://localhost:8000/api/access -d '{"card_id":"test123"}'
# http://localhost:8000/docs (Swagger)
```

## 📚 Documentation
| Section | Link |
|---------|------|
| Architecture/DB | [./docs/architecture.md](./docs/architecture.md) |
| Hardware | [./docs/hardware.md](./docs/hardware.md) |
| Setup | [./docs/setup.md](./docs/setup.md) |
| API | [./docs/api.md](./docs/api.md) |

## 🛠️ Development
- Backend: `cd controller && uvicorn main:app --reload`
- Adapter: `cd rust-adapter && cargo run`
- Docs: Rust `cargo doc --open`, Python Sphinx TBD

## 🤝 Contributing
[./docs/contributing.md](./docs/contributing.md)


### Components
- **rust-adapter/**: Tokio/Warp HTTP API + TCP proxy for Keyence PLC commands (R0/S0 access control, INSPECT). Features shared persistent TCP connections, backend auth checks (JWT), PLC signal handling.
  - Docs: `cd rust-adapter && cargo doc --open`
- **controller/**: Python backend with SQLAlchemy models (User, Card, Policy, AuditLog), API endpoints (/access, /audit), access control logic.
  - DB: PostgreSQL (see `access-control/access-control.sqlproj`).
  - Docs: Add Google-style docstrings → use Sphinx.

## Architecture
```
Keyence PLC ← TCP Proxy (rust-adapter) → HTTP API ← Python Controller (DB + Auth)
Badge scan → Backend check → Grant/Deny PLC signal
```

## Setup & Run
1. **DB**: Apply migrations or `access-control/access-control.sqlproj`.
2. **Python**: `cd controller && pip install -r requirements.txt && uvicorn main:app`.
3. **Rust**: `cd rust-adapter && cargo run`.
4. **Docker**: `docker compose up`.

## Configuration
- `.env`: `KEYENCE_HOST`, `PLC_REGISTER_ALLOW`, `ADAPTER_TOKEN`, DB creds.
- Override: POST `/api/override` with token/passcode.

## API Endpoints (rust-adapter)
- `POST /api/trigger`: PLC signal (ALLOW/DENY).
- `POST /api/authorize-command`: Backend check + forward R0/S0.
- `POST /api/check-access`: Access decision.
- `/health`: OK.

## Development
- Rust docs: Inline `///` → `cargo doc --open`.
- Python: `"""docstrings"""` → `pydocstyle`, Sphinx.
- Logging: Keyence commands to `KEYENCE_COMMAND_LOG`.
- Tests: `cargo test`.

See `rust-adapter/TODO.md` for pending tasks.
