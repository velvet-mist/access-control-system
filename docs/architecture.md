# System Architecture & Data Model

## High-Level Flow
```
RFID Card → Reader → Rust Adapter (Pi) → Python Policy Engine → PostgreSQL (Audit)
                                             ↓ Modbus TCP/RTU
                                       Keyence CV-X480F
```

## Database Schema Summary
**Tables** (PostgreSQL/SQLAlchemy):
- **users**: user_id (UUID PK), display_name, user_type, status, created_at, card_id
- **roles**: role_id (int PK), role_name, description
- **user_roles**: user_id, role_id (composite PK)
- **cards**: card_id (varchar PK), user_id (FK), status, source_system, last_synced_at
- **policies**: policy_id (int PK), role_id (FK), machine_id, command, allow, time_start/end
- **audit_logs**: id (int PK), card_id, user_id, machine_id, command, decision, reason, created_at, adapter_id
- **adapters**: adapter_id (text PK), token, status
- **machines**: machine_id (text PK), machine_type, location, criticality, status
- **override**: override_token (int PK), expires_at, issued_by, reason, active

**Relationships**:
```
User 1⟷N UserRole ⟶1 Role
User 1⟷N Card
User/Card ⟶N AuditLog
Role ⟶N Policy (per machine/command)
```

## Access Flow
1. Card scan → Lookup cards → Get user_id
2. Roles via user_roles → Check policies (role/machine/command/time)
3. Decision logged to audit_logs
4. Forward command/deny to PLC via adapter

See [schema.md](./schema.md) for full DDL.

