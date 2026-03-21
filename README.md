# wsm — Windows Screen Manager

A small CLI tool to save and restore monitor configurations on Windows. Useful for setups where you frequently switch between different display arrangements (e.g., docked vs. undocked, single vs. multi-monitor).

<video src="./demo.mp4" controls title="wsm demo" width="70%"></video>

## Installation

### Scoop (recommended)

```bash
scoop bucket add wsm https://github.com/OsirisFrik/scoop-windows-screen-manager
scoop install wsm
```

### Build from source

Requirements: Windows 10/11 and the [Rust toolchain](https://rustup.rs/).

```bash
cargo build --release
```

The binary will be at `target/release/wsm.exe`.

## Usage

### Save current configuration

Reads all active monitors and writes their settings to a JSON file.

```bash
wsm save                  # saves to config.json (default)
wsm save my-setup.json    # saves to a custom file
```

Example output:

```
Saved 2 monitor(s) to config.json
  Display 1: GS34WQCA                     3440x1440 @ 144Hz  pos (    0,     0)  primary: true
  Display 2: P2419H                       1920x1080 @  60Hz  pos ( 3440,   180)  primary: false
```

### Load a saved configuration

Reads a JSON file and applies all monitor settings atomically.

```bash
wsm load config.json
wsm load my-setup.json
```

Example output:

```
Applying 2 monitor configuration(s) from config.json...
  Display 1: GS34WQCA — 3440x1440 @ 144Hz staged OK
  Display 2: P2419H — 1920x1080 @ 60Hz staged OK
Configuration applied successfully.
```

## What gets saved

Each monitor entry in the JSON file includes:

| Field            | Description                                             |
| ---------------- | ------------------------------------------------------- |
| `device_name`    | Windows adapter name, e.g. `\\.\DISPLAY1`               |
| `friendly_name`  | Human-readable label derived from hardware ID and index |
| `position_x / y` | Virtual desktop position in pixels                      |
| `width / height` | Resolution in pixels                                    |
| `refresh_rate`   | Refresh rate in Hz                                      |
| `bits_per_pel`   | Color depth (typically 32)                              |
| `orientation`    | Rotation: `0`=0°, `1`=90°, `2`=180°, `3`=270°           |
| `is_primary`     | Whether this is the primary display                     |

Example `config.json`:

```json
[
  {
    "device_name": "\\\\.\\DISPLAY1",
    "friendly_name": "Display 1: GS34WQCA",
    "position_x": 0,
    "position_y": 0,
    "width": 3440,
    "height": 1440,
    "refresh_rate": 144,
    "bits_per_pel": 32,
    "orientation": 0,
    "is_primary": true
  }
]
```

## Notes

- Changes are staged per-monitor with `CDS_UPDATEREGISTRY | CDS_NORESET` and committed in a single atomic call, which avoids intermediate invalid display states.
- The primary monitor is always staged first, as Windows requires a valid primary at all times.
- If a monitor from the config file is not currently connected, Windows will silently skip it or return a non-fatal error.

## License

MIT
