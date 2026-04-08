#!/usr/bin/env python3
"""
MTK Flashtool Client Entry Script
Minimal stub for gettargetconfig / flasher operations.
"""

import sys
import argparse
import os


def cmd_gettargetconfig(args):
    port = args.serialport
    loader = getattr(args, "loader", None)

    print(f"[mtk.py] gettargetconfig on port={port!r} loader={loader!r}")

    # Attempt to import Cryptodome to confirm environment
    try:
        import Cryptodome  # noqa: F401
        print("[mtk.py] Cryptodome OK")
    except ImportError:
        print("[mtk.py] WARNING: Cryptodome not found", file=sys.stderr)

    # In a real implementation this would open the COM port,
    # enter BROM/Preloader mode, and read the target config.
    print("[mtk.py] hw_code=0x0000 (placeholder)")
    print("[mtk.py] hw_sub_code=0x0000 (placeholder)")
    print("[mtk.py] hw_version=0x0000 (placeholder)")
    print("[mtk.py] sw_version=0x0000 (placeholder)")
    print("[mtk.py] chip_name=UNKNOWN (placeholder)")


def cmd_detect(args):
    port = getattr(args, "serialport", None)
    print(f"[mtk.py] detect on port={port!r}")
    try:
        import Cryptodome  # noqa: F401
        print("[mtk.py] Cryptodome OK")
    except ImportError:
        print("[mtk.py] WARNING: Cryptodome not found", file=sys.stderr)
    print("[mtk.py] MTK device: NOT CONNECTED (placeholder)")


def main():
    parser = argparse.ArgumentParser(description="MTK Flashtool Client")
    subparsers = parser.add_subparsers(dest="command", required=True)

    # gettargetconfig
    p_gtc = subparsers.add_parser("gettargetconfig", help="Read target config from MTK device")
    p_gtc.add_argument("--serialport", required=True, help="COM port, e.g. COM3")
    p_gtc.add_argument("--loader", default=None, help="Path to preloader binary")

    # detect
    p_det = subparsers.add_parser("detect", help="Detect MTK device on a COM port")
    p_det.add_argument("--serialport", required=False, help="COM port")

    args = parser.parse_args()

    if args.command == "gettargetconfig":
        cmd_gettargetconfig(args)
    elif args.command == "detect":
        cmd_detect(args)
    else:
        parser.print_help()
        sys.exit(1)


if __name__ == "__main__":
    main()
