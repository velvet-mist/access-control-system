# System Diagrams

## Overall Architecture
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  RFID Card      │───▶│   Rust Adapter   │───▶│ Keyence PLC     │
│  (125KHz HID)   │    │   (RPi:8080)     │    │ CV-X480F        │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │
                              ▼ HTTP/API
                       ┌──────────────────┐
                       │  Python Backend  │ ── PostgreSQL (Audit/DB)
                       │  (FastAPI:8000) │
                       └──────────────────┘
```

## RPi GPIO/Power (from original visual.md)
```
RPi GPIO:
3.3V → RFID VCC    5V → LED VCC
GND  → GND         GPIO18 → LED
GPIO14(TX) → RFID RX   GPIO15(RX) → RFID TX
```

Full diagrams/cables: See raw [visual.md](../md files/visual.md).

## Hardware Connections
```
Card → RDM6300(USB) → RPi → USB-TTL → NullModem → CV-X480F RS232
OR
RPi(eth0) ─ Cat6 ─ Switch ─ CV-X480F (Modbus TCP:502)
```

