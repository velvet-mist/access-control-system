# API Reference

## Backend (Python FastAPI:8000)
Auto-docs: http://localhost:8000/docs (Swagger/OpenAPI)

Key endpoints (inferred from routes/access.py, routes/audit.py):

### Access Control
```
POST /api/access
Content-Type: application/json
{
  "card_id": "ABC123...",
  "machine_id": "cvx480f_01",
  "command": "INSPECT"  // or "SETTINGS_CHANGE"
}

Response 200:
{
  "decision": "allow"|"deny",
  "reason": "Policy match",
  "user_id": "uuid",
  "roles": ["operator"]
}
```

### Audit
```
GET /api/audit?card_id=ABC123&machine_id=cv01&from=2024-01-01
Response:
[
  {
    "id": 1,
    "card_id": "...",
    "user_id": "...",
    "decision": "allow",
    "reason": "...",
    "created_at": "..."
  }
]
POST /api/audit (log entry)
```

## Adapter (Rust Warp:8080)
```
POST /api/authorize-command
{
  "command": "R0",  // PLC reg
  "machine_id": "cvx480f"
}
→ Forwards to backend → PLC if allowed

POST /api/trigger { "signal": "ALLOW" }
POST /health → {"status": "ok"}
```

See code: controller/api/routes/, rust-adapter/src/api.rs

