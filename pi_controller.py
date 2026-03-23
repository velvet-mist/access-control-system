#!/usr/bin/env python3
"""
Access Control - Raspberry Pi 3 Controller
==========================================
Reads USB badge scanner and USB mouse events.
Badge scan  → checks access via backend → grants one-shot access
Mouse click → if access granted, toggles CV-X mode (R0/S0) via TCP
              if no access, blocks and logs

Wiring:
  USB mouse   → Pi 3 USB port
  USB badge   → Pi 3 USB port
  Pi 3 eth0   → same network as CV-X

Environment variables (or edit defaults below):
  ADAPTER_URL    - Rust adapter HTTP URL
  ADAPTER_TOKEN  - Bearer token
  KEYENCE_HOST   - CV-X IP
  KEYENCE_PORT   - CV-X command port
  MOUSE_DEVICE   - path to mouse input device
  BADGE_DEVICE   - path to badge reader input device
"""

import asyncio
import logging
import os
import socket
import threading
import time
from enum import Enum

import evdev
from evdev import InputDevice, categorize, ecodes
import httpx

# ── Config ────────────────────────────────────────────────────────────────────

ADAPTER_URL   = os.getenv("ADAPTER_URL",   "http://localhost:8502")
ADAPTER_TOKEN = os.getenv("ADAPTER_TOKEN", "done")
KEYENCE_HOST  = os.getenv("KEYENCE_HOST",  "192.168.0.20")
KEYENCE_PORT  = int(os.getenv("KEYENCE_PORT", "8500"))
MOUSE_DEVICE  = os.getenv("MOUSE_DEVICE",  "")   # auto-detect if empty
BADGE_DEVICE  = os.getenv("BADGE_DEVICE",  "")   # auto-detect if empty
MACHINE_ID    = os.getenv("MACHINE_ID",    "MACHINE_01")

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [PI] %(levelname)s %(message)s",
    datefmt="%H:%M:%S",
)
log = logging.getLogger("pi_controller")

# ── State ─────────────────────────────────────────────────────────────────────

class CvxMode(Enum):
    RUN   = "RUN"
    SETUP = "SETUP"
    UNKNOWN = "UNKNOWN"

class AccessState:
    def __init__(self):
        self._lock       = threading.Lock()
        self._granted    = False
        self._card_id    = None
        self._granted_at = None
        self.ACCESS_TIMEOUT_SECS = 30  # revoke access after 30s if unused

    def grant(self, card_id: str):
        with self._lock:
            self._granted    = True
            self._card_id    = card_id
            self._granted_at = time.time()
        log.info(f"Access GRANTED for card {card_id}")

    def revoke(self):
        with self._lock:
            card = self._card_id
            self._granted    = False
            self._card_id    = None
            self._granted_at = None
        if card:
            log.info(f"Access REVOKED for card {card}")

    def is_granted(self) -> bool:
        with self._lock:
            if not self._granted:
                return False
            # Auto-expire after timeout
            if time.time() - self._granted_at > self.ACCESS_TIMEOUT_SECS:
                log.warning("Access expired (timeout)")
                self._granted    = False
                self._card_id    = None
                self._granted_at = None
                return False
            return True

    def consume(self) -> str | None:
        """Consume access — returns card_id if granted, None if not."""
        with self._lock:
            if not self._granted:
                return None
            if time.time() - self._granted_at > self.ACCESS_TIMEOUT_SECS:
                log.warning("Access expired (timeout)")
                self._granted = False
                self._card_id = None
                return None
            card = self._card_id
            self._granted    = False
            self._card_id    = None
            self._granted_at = None
            return card


class CvxState:
    def __init__(self):
        self._lock = threading.Lock()
        self._mode = CvxMode.UNKNOWN

    def set(self, mode: CvxMode):
        with self._lock:
            self._mode = mode

    def get(self) -> CvxMode:
        with self._lock:
            return self._mode

    def toggle_command(self) -> str:
        """Return the command to switch to the opposite mode."""
        with self._lock:
            if self._mode == CvxMode.RUN:
                return "S0"
            else:
                # SETUP or UNKNOWN — switch to run
                return "R0"


# ── Keyence TCP ───────────────────────────────────────────────────────────────

def send_keyence_command(command: str) -> str | None:
    """Send a command to the CV-X and return the response."""
    try:
        with socket.create_connection((KEYENCE_HOST, KEYENCE_PORT), timeout=3) as sock:
            sock.sendall(f"{command}\r\n".encode())
            response = sock.recv(256).decode().strip()
            log.info(f"Keyence {command} → {response}")
            return response
    except Exception as e:
        log.error(f"Keyence TCP error: {e}")
        return None


# ── Backend API ───────────────────────────────────────────────────────────────

def check_access_backend(card_id: str, command: str) -> bool:
    """Call the Rust adapter to check if this card can execute command."""
    try:
        response = httpx.post(
            f"{ADAPTER_URL}/api/authorize-command",
            json={
                "card_id":    card_id,
                "machine_id": MACHINE_ID,
                "command":    command,
            },
            headers={"Authorization": f"Bearer {ADAPTER_TOKEN}"},
            timeout=5.0,
        )
        data = response.json()
        decision = data.get("decision", "DENY")
        log.info(f"Backend decision for card {card_id} command {command}: {decision}")
        return decision == "ALLOW"
    except Exception as e:
        log.error(f"Backend error: {e}")
        return False


# ── Device auto-detection ─────────────────────────────────────────────────────

def find_device(keyword: str) -> str | None:
    """Find a /dev/input device by name keyword."""
    for path in evdev.list_devices():
        try:
            dev = InputDevice(path)
            if keyword.lower() in dev.name.lower():
                return path
        except Exception:
            continue
    return None


def detect_devices() -> tuple[str | None, str | None]:
    """Auto-detect mouse and badge reader paths."""
    mouse_path = MOUSE_DEVICE or find_device("mouse") or find_device("pointer")
    badge_path = BADGE_DEVICE or find_device("hid") or find_device("reader") or find_device("scanner")

    # If only one device found and it's the same, try to distinguish
    if mouse_path == badge_path:
        badge_path = None

    return mouse_path, badge_path


# ── Badge reader thread ───────────────────────────────────────────────────────

def badge_reader_loop(badge_path: str, access: AccessState, cvx: CvxState):
    """
    Reads badge scans from USB HID keyboard-emulating badge reader.
    Most USB badge readers appear as keyboards and send card_id + Enter.
    """
    log.info(f"Badge reader on {badge_path}")
    dev     = InputDevice(badge_path)
    buffer  = []
    # Grab device so OS doesn't process keystrokes
    dev.grab()

    SHIFT_KEYS = {ecodes.KEY_LEFTSHIFT, ecodes.KEY_RIGHTSHIFT}
    shifted    = False

    KEY_MAP = {
        ecodes.KEY_0: ('0', ')'), ecodes.KEY_1: ('1', '!'),
        ecodes.KEY_2: ('2', '@'), ecodes.KEY_3: ('3', '#'),
        ecodes.KEY_4: ('4', '$'), ecodes.KEY_5: ('5', '%'),
        ecodes.KEY_6: ('6', '^'), ecodes.KEY_7: ('7', '&'),
        ecodes.KEY_8: ('8', '*'), ecodes.KEY_9: ('9', '('),
        ecodes.KEY_A: ('a', 'A'), ecodes.KEY_B: ('b', 'B'),
        ecodes.KEY_C: ('c', 'C'), ecodes.KEY_D: ('d', 'D'),
        ecodes.KEY_E: ('e', 'E'), ecodes.KEY_F: ('f', 'F'),
        ecodes.KEY_G: ('g', 'G'), ecodes.KEY_H: ('h', 'H'),
        ecodes.KEY_I: ('i', 'I'), ecodes.KEY_J: ('j', 'J'),
        ecodes.KEY_K: ('k', 'K'), ecodes.KEY_L: ('l', 'L'),
        ecodes.KEY_M: ('m', 'M'), ecodes.KEY_N: ('n', 'N'),
        ecodes.KEY_O: ('o', 'O'), ecodes.KEY_P: ('p', 'P'),
        ecodes.KEY_Q: ('q', 'Q'), ecodes.KEY_R: ('r', 'R'),
        ecodes.KEY_S: ('s', 'S'), ecodes.KEY_T: ('t', 'T'),
        ecodes.KEY_U: ('u', 'U'), ecodes.KEY_V: ('v', 'V'),
        ecodes.KEY_W: ('w', 'W'), ecodes.KEY_X: ('x', 'X'),
        ecodes.KEY_Y: ('y', 'Y'), ecodes.KEY_Z: ('z', 'Z'),
        ecodes.KEY_MINUS: ('-', '_'), ecodes.KEY_EQUAL: ('=', '+'),
    }

    try:
        for event in dev.read_loop():
            if event.type != ecodes.EV_KEY:
                continue
            key_event = categorize(event)

            if key_event.keycode in [f"KEY_{k}" for k in ["LEFTSHIFT", "RIGHTSHIFT"]]:
                shifted = (key_event.keystate != 0)
                continue

            if key_event.keystate != 1:  # only key down
                continue

            if key_event.scancode == ecodes.KEY_ENTER:
                if buffer:
                    card_id = "".join(buffer)
                    buffer.clear()
                    log.info(f"Badge scanned: {card_id}")

                    # Determine which command we'd need based on current mode
                    command = cvx.toggle_command()

                    # Check access with backend
                    allowed = check_access_backend(card_id, command)
                    if allowed:
                        access.grant(card_id)
                        log.info(f"Badge {card_id} — access granted, waiting for mouse click")
                    else:
                        log.warning(f"Badge {card_id} — access DENIED for {command}")
            else:
                # Accumulate keystrokes into card_id buffer
                code = key_event.scancode
                if code in KEY_MAP:
                    char = KEY_MAP[code][1 if shifted else 0]
                    buffer.append(char)
    except Exception as e:
        log.error(f"Badge reader error: {e}")
    finally:
        try:
            dev.ungrab()
        except Exception:
            pass


# ── Mouse thread ──────────────────────────────────────────────────────────────

def mouse_loop(mouse_path: str, access: AccessState, cvx: CvxState):
    """
    Reads mouse clicks. On left click:
    - If access granted → send toggle command to CV-X, consume access
    - If no access → block and log
    """
    log.info(f"Mouse on {mouse_path}")
    dev = InputDevice(mouse_path)
    dev.grab()  # prevent OS from acting on mouse events

    try:
        for event in dev.read_loop():
            if event.type != ecodes.EV_KEY:
                continue
            if event.code != ecodes.BTN_LEFT:
                continue
            if event.value != 1:  # key down only
                continue

            command = cvx.toggle_command()
            card_id = access.consume()

            if card_id is None:
                log.warning(f"Mouse click blocked — no access granted (would send {command})")
                continue

            log.info(f"Mouse click — sending {command} for card {card_id}")
            response = send_keyence_command(command)

            if response and (response == command or response == "0"):
                # Update tracked mode
                if command == "R0":
                    cvx.set(CvxMode.RUN)
                    log.info("CV-X switched to RUN mode")
                else:
                    cvx.set(CvxMode.SETUP)
                    log.info("CV-X switched to SETUP mode")
            else:
                log.error(f"CV-X rejected {command}: {response}")

    except Exception as e:
        log.error(f"Mouse error: {e}")
    finally:
        try:
            dev.ungrab()
        except Exception:
            pass


# ── Main ──────────────────────────────────────────────────────────────────────

def main():
    log.info("Starting Pi access controller")
    log.info(f"Adapter: {ADAPTER_URL}")
    log.info(f"Keyence: {KEYENCE_HOST}:{KEYENCE_PORT}")

    mouse_path, badge_path = detect_devices()

    if not mouse_path:
        log.error("No mouse device found. Set MOUSE_DEVICE env var or check /dev/input/")
        log.info("Available devices:")
        for path in evdev.list_devices():
            try:
                dev = InputDevice(path)
                log.info(f"  {path}: {dev.name}")
            except Exception:
                pass
        return

    if not badge_path:
        log.error("No badge reader found. Set BADGE_DEVICE env var or check /dev/input/")
        log.info("Available devices:")
        for path in evdev.list_devices():
            try:
                dev = InputDevice(path)
                log.info(f"  {path}: {dev.name}")
            except Exception:
                pass
        return

    log.info(f"Mouse:  {mouse_path}")
    log.info(f"Badge:  {badge_path}")

    # Probe current CV-X mode
    access = AccessState()
    cvx    = CvxState()

    response = send_keyence_command("GS")  # Get Status
    if response:
        log.info(f"CV-X initial status: {response}")
    else:
        log.warning("Could not get CV-X status — assuming RUN mode")
        cvx.set(CvxMode.RUN)

    # Start threads
    badge_thread = threading.Thread(
        target=badge_reader_loop,
        args=(badge_path, access, cvx),
        daemon=True,
        name="badge-reader",
    )
    mouse_thread = threading.Thread(
        target=mouse_loop,
        args=(mouse_path, access, cvx),
        daemon=True,
        name="mouse",
    )

    badge_thread.start()
    mouse_thread.start()

    log.info("Controller running. Scan badge then click mouse to switch CV-X mode.")
    log.info("Press Ctrl+C to stop.")

    try:
        while True:
            time.sleep(1)
            # Watchdog — restart dead threads
            if not badge_thread.is_alive():
                log.warning("Badge thread died — restarting")
                badge_thread = threading.Thread(
                    target=badge_reader_loop,
                    args=(badge_path, access, cvx),
                    daemon=True,
                    name="badge-reader",
                )
                badge_thread.start()
            if not mouse_thread.is_alive():
                log.warning("Mouse thread died — restarting")
                mouse_thread = threading.Thread(
                    target=mouse_loop,
                    args=(mouse_path, access, cvx),
                    daemon=True,
                    name="mouse",
                )
                mouse_thread.start()
    except KeyboardInterrupt:
        log.info("Shutting down")


if __name__ == "__main__":
    main()