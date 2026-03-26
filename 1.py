#!/usr/bin/env python3
"""
RFID Reader Test Script
=======================
Plug in your USB RFID/badge reader and run this script.
Scan a card — it will print the card ID to the terminal.

Requirements:
    pip3 install evdev
"""

import evdev
from evdev import InputDevice, categorize, ecodes

# Key map — USB badge readers act as keyboards
KEY_MAP = {
    ecodes.KEY_0: '0', ecodes.KEY_1: '1', ecodes.KEY_2: '2',
    ecodes.KEY_3: '3', ecodes.KEY_4: '4', ecodes.KEY_5: '5',
    ecodes.KEY_6: '6', ecodes.KEY_7: '7', ecodes.KEY_8: '8',
    ecodes.KEY_9: '9',
    ecodes.KEY_A: 'A', ecodes.KEY_B: 'B', ecodes.KEY_C: 'C',
    ecodes.KEY_D: 'D', ecodes.KEY_E: 'E', ecodes.KEY_F: 'F',
    ecodes.KEY_G: 'G', ecodes.KEY_H: 'H', ecodes.KEY_I: 'I',
    ecodes.KEY_J: 'J', ecodes.KEY_K: 'K', ecodes.KEY_L: 'L',
    ecodes.KEY_M: 'M', ecodes.KEY_N: 'N', ecodes.KEY_O: 'O',
    ecodes.KEY_P: 'P', ecodes.KEY_Q: 'Q', ecodes.KEY_R: 'R',
    ecodes.KEY_S: 'S', ecodes.KEY_T: 'T', ecodes.KEY_U: 'U',
    ecodes.KEY_V: 'V', ecodes.KEY_W: 'W', ecodes.KEY_X: 'X',
    ecodes.KEY_Y: 'Y', ecodes.KEY_Z: 'Z',
    ecodes.KEY_MINUS: '-', ecodes.KEY_EQUAL: '=',
}

def list_devices():
    devices = evdev.list_devices()
    if not devices:
        print("No input devices found.")
        return []
    print("Available input devices:")
    for path in devices:
        try:
            dev = InputDevice(path)
            print(f"  {path}  →  {dev.name}")
        except Exception:
            pass
    return devices

def find_badge_reader():
    """Auto-detect badge reader — looks for HID/keyboard devices."""
    for path in evdev.list_devices():
        try:
            dev = InputDevice(path)
            name = dev.name.lower()
            # Badge readers typically show up as HID or keyboard devices
            if any(k in name for k in ['hid', 'keyboard', 'reader', 'scanner', 'barcode', 'rfid']):
                return path, dev.name
        except Exception:
            continue
    return None, None

def read_card(device_path: str):
    """Read card IDs from the badge reader indefinitely."""
    dev = InputDevice(device_path)
    print(f"\nListening on: {dev.name} ({device_path})")
    print("Scan a card...\n")

    # Grab device so keystrokes don't go to terminal
    dev.grab()
    buffer = []

    try:
        for event in dev.read_loop():
            if event.type != ecodes.EV_KEY:
                continue

            key_event = categorize(event)
            if key_event.keystate != 1:  # key down only
                continue

            if key_event.scancode == ecodes.KEY_ENTER:
                if buffer:
                    card_id = "".join(buffer)
                    print(f"Card ID: {card_id}")
                    buffer.clear()
            elif key_event.scancode in KEY_MAP:
                buffer.append(KEY_MAP[key_event.scancode])

    except KeyboardInterrupt:
        print("\nStopped.")
    finally:
        dev.ungrab()

def main():
    print("=== RFID Reader Test ===\n")

    # List all devices
    list_devices()

    # Try auto-detect
    path, name = find_badge_reader()

    if path:
        print(f"\nAuto-detected badge reader: {name} ({path})")
        use_auto = input("Use this device? [Y/n]: ").strip().lower()
        if use_auto in ('', 'y', 'yes'):
            read_card(path)
            return

    # Manual selection
    print("\nEnter device path manually (e.g. /dev/input/event0):")
    path = input("> ").strip()
    if path:
        read_card(path)
    else:
        print("No device selected. Exiting.")

if __name__ == "__main__":
    main()