#  254MTK

A Windows-first Rust CLI that bundles a Python 3.12 runtime and exposes
read-only MTK Preloader/BROM commands via `flashtool`.

## Workspace layout

```
crates/
  flashtool-core/     — shared Error/Result types
  flashtool-mtk/      — MTK logic: Python runner, loader resolution, commands
  flashtool-cli/      — clap-based binary (flashtool)
```

## Build

```powershell
cargo build
```

> **Note**: A placeholder Python runtime is embedded by default.  The binary
> will compile but commands will fail until you generate the real runtime zip
> (see below).

## Build the bundled Python runtime (required for `runner diagnose`)

The `flashtool` binary embeds a Python 3.12 environment that is extracted to
`%TEMP%\flashtool\pyenv\` on first use.  The repository ships a tiny
placeholder zip so `cargo build` succeeds without committing a large binary.
Before running any command you must generate the real zip locally.

### Prerequisites

- Windows 10/11 (x64)
- PowerShell 5.1 or later (included with Windows)
- Internet access (~15 MB download from python.org and PyPI)

### Steps

1. **Generate the zip** (from the repo root in PowerShell):

   ```powershell
   .\scripts\build_py312_zip.ps1
   ```

   The script downloads Python 3.12 x64 embeddable, patches `python312._pth`
   to enable `site-packages`, installs **pycryptodome**, validates the import,
   and writes `crates/flashtool-mtk/resources/py/py312-win64.zip`.

2. **Prevent accidental git commits** of the generated zip:

   ```powershell
   git update-index --assume-unchanged crates/flashtool-mtk/resources/py/py312-win64.zip
   ```

3. **Rebuild** to embed the new zip:

   ```powershell
   cargo build
   ```

4. **Verify**:

   ```powershell
   cargo run -p flashtool-cli -- runner diagnose
   ```

   Expected output:
   ```
   Python path : %TEMP%\flashtool\pyenv\python312\python.exe
   Python found: true
   Cryptodome  : Cryptodome OK
   mtk.py      : %TEMP%\flashtool\pyenv\mtk.py
   ```

See [`scripts/README.md`](scripts/README.md) for full script usage and options.

## USB / libusb driver setup (required for device commands)

MTK devices communicate over USB, but Windows presents them differently
depending on the device mode:

| Mode | Windows driver | Port seen |
|------|---------------|-----------|
| **VCOM** (booted ROM) | CDC Serial / USB Serial | `COM3`, `COM4`, … |
| **BROM** (power-off + Vol↓) | *none by default* → must install WinUSB | raw USB — no COM port |

Most `flashtool` commands target **BROM mode**.  You must replace the default
Windows driver with **WinUSB** using **Zadig** before the tool can open the
device.

### Step-by-step: install WinUSB via Zadig

1. **Download Zadig** from <https://zadig.akeo.ie> (no install needed, portable `.exe`).

2. **Enter BROM mode** on your device:
   - Power the device **off** completely (remove battery if possible).
   - Hold **Volume Down** (some devices: Vol↓ + Vol↑ simultaneously).
   - Plug the USB cable into the PC while holding the button.
   - The device should appear in Device Manager as **"MediaTek USB Port"** or
     **"MTK Preloader"** (VID `0E8D`, PID `0003` or `2000`).

3. **Open Zadig** and choose **Options → List All Devices**.

4. In the device drop-down, select the MediaTek entry (e.g.
   `MTK USB Port (Interface 0)` or `MediaTek Preloader`).
   Confirm the USB ID shows `0E8D:0003` or `0E8D:2000`.

5. Set the driver on the **right** side to **WinUSB (v6.1.xxxx.xxxxx)**.

6. Click **Replace Driver** (or **Install Driver**) and wait for Zadig to finish.

7. **Verify**: open Device Manager → **Universal Serial Bus devices** → you
   should see the MediaTek entry listed there (not under Ports/COM/LPT).

> **Reverting**: to restore the original driver, open Device Manager, right-click
> the device → *Update driver → Browse my computer → Let me pick* → choose
> **USB Serial Device (CDC)**.

### After installing the driver

With WinUSB installed the device has no COM port number.  Pass the device
by specifying the default port placeholder — `mtk.py` resolves the USB
device directly via libusb:

```powershell
flashtool detect --port COM3         # still required by the CLI flag; mtk.py
flashtool read-info --port COM3      # opens the USB device, not the COM port
```

> On **Linux** no driver swap is needed.  Add a udev rule so you can access
> the device without root:
> ```bash
> echo 'SUBSYSTEM=="usb", ATTR{idVendor}=="0e8d", MODE="0666"' \
>   | sudo tee /etc/udev/rules.d/99-mtk.rules
> sudo udevadm control --reload-rules && sudo udevadm trigger
> ```
> Pass the Linux serial device with `--port /dev/ttyUSB0` (VCOM) or omit
> the port flag to let mtk.py auto-discover the USB device.

## CLI commands

```
flashtool runner diagnose          # verify Python env + Cryptodome
flashtool detect --port COM3       # detect MTK device on serial port
flashtool read-info --port COM3    # read target config (gettargetconfig)
```

## Resources

- Placeholder `py312-win64.zip` and `preloader_kl5_xk678.bin` are committed
  only as build stubs.  Replace with real assets (using the provided script)
  before device testing.
- Do **not** commit the generated `py312-win64.zip` to git.
