# wsm — Windows Screen Manager

A small CLI tool to save and restore monitor configurations on Windows. Useful for setups where you frequently switch between different display arrangements (e.g., docked vs. undocked, single vs. multi-monitor).

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

### Save current configuration with a profile alias

Reads all active monitors and saves them to `~/.wsm-profiles` with a profile alias (default YAML format).

```bash
wsm save office           # save as "office" profile
wsm save office --json    # save as "office" profile in JSON format
wsm save office --output my-config.yaml         # save to custom file (YAML)
wsm save office --output my-config.json --json  # save to custom file (JSON)
```

Example output:

```
Saved 2 monitor(s) to profile 'office' in ~/.wsm-profiles (YAML)
  Display 1: GS34WQCA                     3440x1440 @ 144Hz  pos (    0,     0)  primary: true
  Display 2: P2419H                       1920x1080 @  60Hz  pos ( 3440,   180)  primary: false
```

### Load a saved profile

Applies a saved profile configuration atomically.

```bash
wsm load office                         # load profile from ~/.wsm-profiles
wsm load office --source my-config.yaml # load profile from external file
```

Example output:

```
Applying 2 monitor configuration(s) from profile 'office'...
  Display 1: GS34WQCA — 3440x1440 @ 144Hz staged OK
  Display 2: P2419H — 1920x1080 @ 60Hz staged OK
Configuration applied successfully.
```

### List all profiles

```bash
wsm list
```

Example output:

```
Saved profiles in ~/.wsm-profiles:
  office — 2 monitor(s)
    Display 1: GS34WQCA                     3440x1440 @ 144Hz  pos (    0,     0)  primary: true
    Display 2: P2419H                       1920x1080 @  60Hz  pos ( 3440,   180)  primary: false
  home — 1 monitor(s)
    Display 1: P2419H                       1920x1080 @  60Hz  pos (    0,     0)  primary: true
```

### Delete a profile

```bash
wsm delete office
```

## What gets saved

Profiles are stored in `~/.wsm-profiles` (YAML by default, or JSON with `--json` flag).

Each monitor entry includes:

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

Example YAML format (`.wsm-profiles`):

```yaml
profiles:
  office:
    monitors:
      - device_name: '\\.\DISPLAY1'
        friendly_name: 'Display 1: GS34WQCA'
        position_x: 0
        position_y: 0
        width: 3440
        height: 1440
        refresh_rate: 144
        bits_per_pel: 32
        orientation: 0
        is_primary: true
      - device_name: '\\.\DISPLAY2'
        friendly_name: 'Display 2: P2419H'
        position_x: 3440
        position_y: 180
        width: 1920
        height: 1080
        refresh_rate: 60
        bits_per_pel: 32
        orientation: 0
        is_primary: false
```

## Notes

- Changes are staged per-monitor with `CDS_UPDATEREGISTRY | CDS_NORESET` and committed in a single atomic call, which avoids intermediate invalid display states.
- The primary monitor is always staged first, as Windows requires a valid primary at all times.
- If a monitor from the config file is not currently connected, Windows will silently skip it or return a non-fatal error.

## License

MIT
