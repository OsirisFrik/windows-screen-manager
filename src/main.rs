use std::mem;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use windows::Win32::Graphics::Gdi::{
    ChangeDisplaySettingsExW, EnumDisplayDevicesW, EnumDisplaySettingsExW, CDS_NORESET,
    CDS_SET_PRIMARY, CDS_TYPE, CDS_UPDATEREGISTRY, DEVMODEW, DEVMODE_DISPLAY_ORIENTATION,
    DISP_CHANGE_SUCCESSFUL, DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICE_PRIMARY_DEVICE,
    DISPLAY_DEVICEW, DM_BITSPERPEL, DM_DISPLAYFREQUENCY, DM_DISPLAYORIENTATION, DM_PELSHEIGHT,
    DM_PELSWIDTH, DM_POSITION, ENUM_CURRENT_SETTINGS, ENUM_DISPLAY_SETTINGS_FLAGS,
};
use windows::core::PCWSTR;

#[derive(Parser)]
#[command(name = "wsm")]
#[command(about = "Windows Screen Manager — save and restore monitor configurations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Read current display configuration and save it to a JSON file
    Save {
        /// Output JSON file (default: config.json)
        #[arg(default_value = "config.json")]
        output: PathBuf,
    },
    /// Apply a display configuration from a JSON file
    Load {
        /// JSON configuration file to apply
        config: PathBuf,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct MonitorConfig {
    device_name: String,
    friendly_name: Option<String>,
    position_x: i32,
    position_y: i32,
    width: u32,
    height: u32,
    refresh_rate: u32,
    bits_per_pel: u32,
    /// Raw value of DEVMODE_DISPLAY_ORIENTATION (0=0°, 1=90°, 2=180°, 3=270°)
    orientation: u32,
    is_primary: bool,
}

fn wide_to_string(wide: &[u16]) -> String {
    let end = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..end])
}

fn to_wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Calls EnumDisplayDevicesW with the adapter name to get the attached monitor's model.
/// DeviceID format: "MONITOR\GS34WQCA\{4d36e96e-...}\0041" — the second segment is
/// the hardware ID that Windows Settings displays as the monitor name.
fn get_monitor_model(adapter_device_name: &str) -> Option<String> {
    let wide = to_wide_null(adapter_device_name);
    let mut monitor = DISPLAY_DEVICEW {
        cb: mem::size_of::<DISPLAY_DEVICEW>() as u32,
        ..Default::default()
    };
    let found =
        unsafe { EnumDisplayDevicesW(PCWSTR(wide.as_ptr()), 0, &mut monitor, 0) };
    if !found.as_bool() {
        return None;
    }
    // DeviceID: "MONITOR\<MODEL_ID>\{GUID}\<instance>"
    let device_id = wide_to_string(&monitor.DeviceID);
    let model = device_id.split('\\').nth(1)?;
    if model.is_empty() {
        return None;
    }
    Some(model.to_string())
}

/// Extracts the display index from "\\.\DISPLAY3" → 3.
fn display_index(device_name: &str) -> Option<u32> {
    device_name
        .to_uppercase()
        .rsplit("DISPLAY")
        .next()
        .and_then(|s| s.parse().ok())
}

fn enumerate_monitors() -> Vec<MonitorConfig> {
    let mut monitors = Vec::new();
    let mut dev_num = 0u32;

    loop {
        let mut display_device = DISPLAY_DEVICEW {
            cb: mem::size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };

        let found =
            unsafe { EnumDisplayDevicesW(PCWSTR::null(), dev_num, &mut display_device, 0) };

        if !found.as_bool() {
            break;
        }
        dev_num += 1;

        // Skip adapters not connected to the desktop
        if display_device.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP == 0 {
            continue;
        }

        let device_name = wide_to_string(&display_device.DeviceName);
        let device_name_wide = to_wide_null(&device_name);

        let mut devmode = DEVMODEW {
            dmSize: mem::size_of::<DEVMODEW>() as u16,
            ..Default::default()
        };

        let ok = unsafe {
            EnumDisplaySettingsExW(
                PCWSTR(device_name_wide.as_ptr()),
                ENUM_CURRENT_SETTINGS,
                &mut devmode,
                ENUM_DISPLAY_SETTINGS_FLAGS(0),
            )
        };

        if !ok.as_bool() {
            eprintln!("Warning: could not read settings for {device_name}");
            continue;
        }

        let (pos_x, pos_y, orientation) = unsafe {
            let anon = &devmode.Anonymous1.Anonymous2;
            (anon.dmPosition.x, anon.dmPosition.y, anon.dmDisplayOrientation.0)
        };

        let friendly_name = {
            let model = get_monitor_model(&device_name);
            let index = display_index(&device_name);
            match (index, model) {
                (Some(i), Some(m)) => Some(format!("Display {i}: {m}")),
                (Some(i), None) => Some(format!("Display {i}")),
                (None, Some(m)) => Some(m),
                (None, None) => None,
            }
        };

        monitors.push(MonitorConfig {
            device_name,
            friendly_name,
            position_x: pos_x,
            position_y: pos_y,
            width: devmode.dmPelsWidth,
            height: devmode.dmPelsHeight,
            refresh_rate: devmode.dmDisplayFrequency,
            bits_per_pel: devmode.dmBitsPerPel,
            orientation,
            is_primary: display_device.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE != 0,
        });
    }

    monitors
}

fn save_config(output: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let monitors = enumerate_monitors();

    if monitors.is_empty() {
        return Err("No active monitors found.".into());
    }

    let json = serde_json::to_string_pretty(&monitors)?;
    std::fs::write(output, &json)?;

    println!("Saved {} monitor(s) to {}", monitors.len(), output.display());
    for m in &monitors {
        let label = m.friendly_name.as_deref().unwrap_or(&m.device_name);
        println!(
            "  {:<40} {}x{} @ {}Hz  pos ({:>5}, {:>5})  primary: {}",
            label, m.width, m.height, m.refresh_rate, m.position_x, m.position_y, m.is_primary
        );
    }

    Ok(())
}

fn load_config(config: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let json = std::fs::read_to_string(config)?;
    let mut monitors: Vec<MonitorConfig> = serde_json::from_str(&json)?;

    // Primary monitor must be staged first — Windows requires a valid primary at all times
    monitors.sort_by_key(|m| !m.is_primary);

    println!("Applying {} monitor configuration(s) from {}...", monitors.len(), config.display());

    for monitor in &monitors {
        let device_name_wide = to_wide_null(&monitor.device_name);
        let label = monitor.friendly_name.as_deref().unwrap_or(&monitor.device_name);

        let mut devmode = DEVMODEW {
            dmSize: mem::size_of::<DEVMODEW>() as u16,
            dmFields: DM_BITSPERPEL
                | DM_PELSWIDTH
                | DM_PELSHEIGHT
                | DM_DISPLAYFREQUENCY
                | DM_POSITION
                | DM_DISPLAYORIENTATION,
            dmBitsPerPel: monitor.bits_per_pel,
            dmPelsWidth: monitor.width,
            dmPelsHeight: monitor.height,
            dmDisplayFrequency: monitor.refresh_rate,
            ..Default::default()
        };

        unsafe {
            let anon = &mut devmode.Anonymous1.Anonymous2;
            anon.dmPosition.x = monitor.position_x;
            anon.dmPosition.y = monitor.position_y;
            anon.dmDisplayOrientation = DEVMODE_DISPLAY_ORIENTATION(monitor.orientation);
        }

        // CDS_SET_PRIMARY tells Windows which monitor becomes the new primary
        let flags = if monitor.is_primary {
            CDS_UPDATEREGISTRY | CDS_NORESET | CDS_SET_PRIMARY
        } else {
            CDS_UPDATEREGISTRY | CDS_NORESET
        };

        let result = unsafe {
            ChangeDisplaySettingsExW(
                PCWSTR(device_name_wide.as_ptr()),
                Some(&devmode),
                None,
                flags,
                None,
            )
        };

        if result == DISP_CHANGE_SUCCESSFUL {
            println!(
                "  {} — {}x{} @ {}Hz staged OK",
                label, monitor.width, monitor.height, monitor.refresh_rate
            );
        } else {
            eprintln!("  Warning: staging failed for {} (code {})", label, result.0);
        }
    }

    // Commit all staged changes in one shot
    let result =
        unsafe { ChangeDisplaySettingsExW(PCWSTR::null(), None, None, CDS_TYPE(0), None) };

    if result != DISP_CHANGE_SUCCESSFUL {
        return Err(format!("Failed to commit display changes (code {})", result.0).into());
    }

    println!("Configuration applied successfully.");
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Save { output } => save_config(&output),
        Commands::Load { config } => load_config(&config),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
