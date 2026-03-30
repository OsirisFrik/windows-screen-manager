use std::mem;
use std::path::Path;

use windows::Win32::Graphics::Gdi::{
    ChangeDisplaySettingsExW, CDS_NORESET, CDS_SET_PRIMARY, CDS_TYPE, CDS_UPDATEREGISTRY, DEVMODEW,
    DEVMODE_DISPLAY_ORIENTATION, DISP_CHANGE_SUCCESSFUL, DM_BITSPERPEL, DM_DISPLAYFREQUENCY,
    DM_DISPLAYORIENTATION, DM_PELSHEIGHT, DM_PELSWIDTH, DM_POSITION,
};
use windows::core::PCWSTR;

use crate::display::{enumerate_monitors, to_wide_null};
use crate::profiles;
use crate::types::{Config, MonitorConfig};

fn apply_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut monitors: Vec<MonitorConfig> = config.monitors.clone();

    // Primary monitor must be staged first — Windows requires a valid primary at all times
    monitors.sort_by_key(|m| !m.is_primary);

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
    let result = unsafe { ChangeDisplaySettingsExW(PCWSTR::null(), None, None, CDS_TYPE(0), None) };

    if result != DISP_CHANGE_SUCCESSFUL {
        return Err(format!("Failed to commit display changes (code {})", result.0).into());
    }

    println!("Configuration applied successfully.");
    Ok(())
}

pub fn save_profile(alias: &str, output: Option<&Path>, as_json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let monitors = enumerate_monitors();

    if monitors.is_empty() {
        return Err("No active monitors found.".into());
    }

    let config = Config { monitors };

    if let Some(file) = output {
        profiles::save_profile_to_file(alias, config.clone(), file, as_json)?;
        let fmt = if as_json { "JSON" } else { "YAML" };
        println!(
            "Saved {} monitor(s) to profile '{}' in {} ({})",
            config.monitors.len(),
            alias,
            file.display(),
            fmt
        );
    } else {
        profiles::save_profile(alias, config.clone(), as_json)?;
        let fmt = if as_json { "JSON" } else { "YAML" };
        println!("Saved {} monitor(s) to profile '{}' in ~/.wsm-profiles ({})", config.monitors.len(), alias, fmt);
    }

    for m in &config.monitors {
        let label = m.friendly_name.as_deref().unwrap_or(&m.device_name);
        println!(
            "  {:<40} {}x{} @ {}Hz  pos ({:>5}, {:>5})  primary: {}",
            label, m.width, m.height, m.refresh_rate, m.position_x, m.position_y, m.is_primary
        );
    }

    Ok(())
}

pub fn load_profile(
    alias: &str,
    source: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = if let Some(source_path) = source {
        // Load from external file
        let contents = std::fs::read_to_string(source_path)?;
        let ext = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let profiles: crate::profiles::Profiles = match ext.as_str() {
            "json" => serde_json::from_str(&contents)?,
            _ => serde_yaml::from_str(&contents)?,
        };

        profiles
            .profiles
            .get(alias)
            .cloned()
            .ok_or_else(|| format!("Profile '{}' not found in {}", alias, source_path.display()))?
    } else {
        // Load from .wsm-profiles
        profiles::get_profile(alias)?
    };

    println!(
        "Applying {} monitor configuration(s) from profile '{}'...",
        config.monitors.len(),
        alias
    );

    apply_config(&config)?;
    Ok(())
}

pub fn list_profiles() -> Result<(), Box<dyn std::error::Error>> {
    let profiles = profiles::list_profiles()?;

    if profiles.profiles.is_empty() {
        println!("No profiles saved yet.");
        return Ok(());
    }

    println!("Saved profiles in ~/.wsm-profiles:");
    for (alias, config) in &profiles.profiles {
        println!("  {} — {} monitor(s)", alias, config.monitors.len());
        for m in &config.monitors {
            let label = m.friendly_name.as_deref().unwrap_or(&m.device_name);
            println!(
                "    {:<40} {}x{} @ {}Hz  pos ({:>5}, {:>5})  primary: {}",
                label, m.width, m.height, m.refresh_rate, m.position_x, m.position_y, m.is_primary
            );
        }
    }

    Ok(())
}

pub fn delete_profile(alias: &str) -> Result<(), Box<dyn std::error::Error>> {
    profiles::delete_profile(alias)?;
    println!("Profile '{}' deleted.", alias);
    Ok(())
}
