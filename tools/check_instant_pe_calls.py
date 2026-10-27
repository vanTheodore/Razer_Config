#!/usr/bin/env python3
"""Auditoría pasiva de destinos CALL rel32 del PE Instant. No ejecuta el EXE."""
import hashlib
from pathlib import Path
import struct
import sys

SHA256 = "be9d93115074696eaa4d1f382c6b98e0109baea3a53f7482e1adc880a3a865ed"
SITES = {
    0x41B54E: "posible lectura inicial, 32 bytes",
    0x41B593: "posible llamada de 8 bytes",
    0x41B6A3: "otra llamada en arranque",
    0x4273F8: "configuración RGB, ruta A",
    0x430609: "configuración RGB, ruta B",
}

def u16(buf, pos):
    return struct.unpack_from("<H", buf, pos)[0]

def u32(buf, pos):
    return struct.unpack_from("<I", buf, pos)[0]

def parse_sections(buf):
    if buf[:2] != b"MZ":
        raise ValueError("No hay cabecera MZ")
    header = u32(buf, 0x3C)
    if buf[header:header + 4] != b"PE\\0\\0" or u16(buf, header + 24) != 0x10B:
        raise ValueError("Se requiere un PE32 x86")
    base = u32(buf, header + 24 + 28)
    section_table = header + 24 + u16(buf, header + 20)
    items = []
    for i in range(u16(buf, header + 6)):
        pos = section_table + 40 * i
        name = buf[pos:pos + 8].split(b"\\0")[0].decode("ascii", errors="replace")
        size, rva, raw_size, raw_offset = struct.unpack_from("<IIII", buf, pos + 8)
        flags = u32(buf, pos + 36)
        items.append((name, base + rva, size, raw_offset, raw_size, flags))
    return items

def locate(va, sections):
    for name, base, length, raw, size, flags in sections:
        if base <= va < base + length:
            off = va - base
            return name, raw + off if off < size else None, bool(flags & 0x20000000)
    return "sin sección", None, False

def audit(path):
    buf = Path(path).read_bytes()
    digest = hashlib.sha256(buf).hexdigest()
    print("SHA256:", digest)
    if digest != SHA256:
        raise ValueError("SHA-256 inesperado; no interpretar estas direcciones")
    sections = parse_sections(buf)
    for name, base, length, raw, size, flags in sections:
        print(f"{name}: 0x{base:08X}..0x{base+length-1:08X}, exec={bool(flags & 0x20000000)}")
    for src, desc in SITES.items():
        section, offset, _ = locate(src, sections)
        if offset is None or buf[offset] != 0xE8:
            raise ValueError(f"En 0x{src:X} se esperaba CALL E8")
        rel = struct.unpack_from("<i", buf, offset + 1)[0]
        dest = src + 5 + rel
        target_sec, target_off, executable = locate(dest, sections)
        raw_bytes = buf[target_off:target_off+8].hex(" ") if target_off is not None else "N/A"
        print(f"0x{src:08X} ({desc}) -> 0x{dest:08X} [{target_sec}, exec={executable}], bytes={raw_bytes}")
    print("Import thunks HID conocidos: GetFeature=0x55F6B4; SetFeature=0x55F6BA")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("Uso: python check_instant_pe_calls.py exe_extraído.exe")
    audit(sys.argv[1])
