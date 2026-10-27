#!/usr/bin/env python3
"""Restore the Inno Setup 5.3.9+ x86 CALL/JMP compression filter.

Method independently adapted from Daniel Scharrer's innoextract
src/stream/exefilter.hpp (2011-2019, zlib license).
This script NEVER executes the supplied Windows program.
"""
import argparse
import hashlib
from pathlib import Path
import struct

SAMPLE_SHA256 = "4488e47491823de6746564c0a0ec0063ff05328d3d3776200318e45117b1e358"
SAMPLE_AUTHENTICODE = "1445ab08184e1cc7d2ec9342195ed216d51c1f11026fee8f9c61b0e55b0d2591"


def decode_5309(data: bytes) -> tuple[bytes, int]:
    out = bytearray(data)
    i = 0
    changed = 0
    while i + 5 <= len(out):
        if out[i] not in (0xE8, 0xE9) or 0x10000 - (i % 0x10000) < 5:
            i += 1
            continue
        if out[i + 4] in (0x00, 0xFF):
            addr = out[i + 1] | (out[i + 2] << 8) | (out[i + 3] << 16)
            rel = (addr - ((i + 5) & 0xFFFFFF)) & 0xFFFFFF
            high = out[i + 4] ^ (0xFF if rel & 0x800000 else 0)
            out[i + 1:i + 5] = bytes((rel & 255, (rel >> 8) & 255,
                                      (rel >> 16) & 255, high))
            changed += 1
        i += 5  # The next 4 bytes are the operand, not fresh opcodes.
    return bytes(out), changed


def authenticode_sha256(pe: bytes) -> str:
    if pe[:2] != b"MZ":
        raise ValueError("Cabecera MZ ausente")
    h = struct.unpack_from("<I", pe, 0x3C)[0]
    if pe[h:h + 4] != b"PE\0\0":
        raise ValueError("No es un PE")
    opt = h + 24
    checksum = opt + 64
    cert_entry = opt + 128  # OptionalHeader32: DataDirectory[4]
    cert_off, cert_size = struct.unpack_from("<II", pe, cert_entry)
    if cert_off + cert_size > len(pe):
        raise ValueError("El certificado Authenticode está incompleto")
    d = hashlib.sha256()
    d.update(pe[:checksum])
    d.update(pe[checksum + 4:cert_entry])
    d.update(pe[cert_entry + 8:cert_off])
    d.update(pe[cert_off + cert_size:])
    return d.hexdigest()


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("stream", type=Path, help="corriente de archivos Inno ya descomprimida")
    ap.add_argument("-o", "--output", type=Path, required=True)
    ap.add_argument("--offset", type=lambda s: int(s, 0), default=0x16C81)
    ap.add_argument("--length", type=lambda s: int(s, 0), default=0x254010)
    args = ap.parse_args()
    stream = args.stream.read_bytes()
    source = stream[args.offset:args.offset + args.length]
    if len(source) != args.length:
        raise ValueError("La corriente no incluye todo el PE y su certificado")
    restored, n = decode_5309(source)
    got_sha = hashlib.sha256(restored).hexdigest()
    got_auth = authenticode_sha256(restored)
    print("Operandos restaurados:", n)
    print("SHA256 PE:", got_sha)
    print("Digest Authenticode:", got_auth)
    if (got_sha, got_auth) != (SAMPLE_SHA256, SAMPLE_AUTHENTICODE):
        raise ValueError("SHA256 inesperado: archivo, rango o algoritmo diferente. No se guarda.")
    args.output.write_bytes(restored)
    print("PE restaurado guardado en", args.output, "(no se ejecutó)")


if __name__ == "__main__":
    main()
