use std::mem;

use windows::Win32::Graphics::Gdi::{
    EnumDisplayDevicesW, EnumDisplaySettingsExW, DEVMODEW, DISPLAY_DEVICE_ATTACHED_TO_DESKTOP,
    DISPLAY_DEVICE_PRIMARY_DEVICE, DISPLAY_DEVICEW, ENUM_CURRENT_SETTINGS,
    ENUM_DISPLAY_SETTINGS_FLAGS,
};
use windows::core::PCWSTR;

use crate::types::MonitorConfig;

pub fn wide_to_string(wide: &[u16]) -> String {
    let end = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..end])
}

pub fn to_wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Calls EnumDisplayDevicesW with the adapter name to get the attached monitor's model.
/// DeviceID format: "MONITOR\GS34WQCA\{4d36e96e-...}\0041" — the second segment is
/// the hardware ID that Windows Settings displays as the monitor name.
pub fn get_monitor_model(adapter_device_name: &str) -> Option<String> {
    let wide = to_wide_null(adapter_device_name);
    let mut monitor = DISPLAY_DEVICEW {
        cb: mem::size_of::<DISPLAY_DEVICEW>() as u32,
        ..Default::default()
    };
    let found = unsafe { EnumDisplayDevicesW(PCWSTR(wide.as_ptr()), 0, &mut monitor, 0) };
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
pub fn display_index(device_name: &str) -> Option<u32> {
    device_name
        .to_uppercase()
        .rsplit("DISPLAY")
        .next()
        .and_then(|s| s.parse().ok())
}

pub fn enumerate_monitors() -> Vec<MonitorConfig> {
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
