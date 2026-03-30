# Database Schema

Extracted from `md files/access_control_sql.docx`.

## Tables

### adapters
| Column | Type | PK | Nullable |
|--------|------|----|----------|
| adapter_id | text | Yes | No |
| token | text | | No |
| status | text | | Yes |

### audit_logs
| Column | Type | PK | Nullable |
|--------|------|----|----------|
| id | integer | Yes | No |
| card_id | varchar | | Yes |
| user_id | varchar | | Yes |
| machine_id | varchar | | Yes |
| command | varchar | | No |
| decision | varchar | | No |
| reason | varchar | | Yes |
| created_at | timestamptz | | Yes (now()) |
| adapter_id | text | | Yes |

### cards
| Column | Type | PK | Nullable |
|--------|------|----|----------|
| card_id | varchar(64) | Yes | No |
| user_id | uuid | | Yes |
| status | text | | Yes |
| source_system | text | | Yes |
| last_synced_at | timestamp | | Yes |

*(Similar for machines, override, policies, roles, user_roles, users - full in original .docx)*

See [architecture.md](./architecture.md) for relationships/flow.

