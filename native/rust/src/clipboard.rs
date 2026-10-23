//! HarmonyOS clipboard transport for PhotoCraft's RGBA8 image clipboard and text editing.
#![cfg_attr(not(target_env = "ohos"), allow(dead_code))]

use crate::bridge;

const MAX_IMAGE_BYTES: usize = 120 * 1024 * 1024;

fn image_byte_len(width: u32, height: u32) -> Result<usize, String> {
    let size = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4));
    match size {
        Some(size) if width > 0 && height > 0 && size <= MAX_IMAGE_BYTES => Ok(size),
        _ => Err("Clipboard image is empty or exceeds 120 MiB".into()),
    }
}

fn decode_image(metadata: &str, pixels: Vec<u8>) -> Result<(u32, u32, Vec<u8>), String> {
    let [width, height]: [u32; 2] =
        serde_json::from_str(metadata).map_err(|error| error.to_string())?;
    if pixels.len() != image_byte_len(width, height)? {
        return Err("Clipboard RGBA buffer has an invalid length".into());
    }
    Ok((width, height, pixels))
}

fn report_error(error: &str) {
    if error != "cancelled" {
        bridge::notify(0, "error", error, &[]);
    }
}

fn report_diagnostic(error: &str) {
    if error != "cancelled" {
        bridge::notify(0, "diagnostic", error, &[]);
    }
}

pub fn set_image(width: u32, height: u32, pixels: &[u8]) -> Result<(), String> {
    let result = (|| {
        if pixels.len() != image_byte_len(width, height)? {
            return Err("Clipboard RGBA buffer has an invalid length".into());
        }
        bridge::request(
            "clipboard_set_image",
            &serde_json::json!([width, height]).to_string(),
            pixels,
        )
        .map(|_| ())
    })();
    if let Err(error) = &result {
        report_error(error);
    }
    result
}

pub fn get_image() -> Result<Option<(u32, u32, Vec<u8>)>, String> {
    match bridge::request("clipboard_get_image", "", &[]) {
        Ok((metadata, _)) if metadata.is_empty() => Ok(None),
        Ok((metadata, pixels)) => match decode_image(&metadata, pixels) {
            Ok(image) => Ok(Some(image)),
            Err(error) => {
                report_diagnostic(&error);
                Err(error)
            }
        },
        Err(error) => {
            report_diagnostic(&error);
            Err(error)
        }
    }
}

pub fn set_text(text: &str) -> Result<(), String> {
    let result = bridge::request("clipboard_set_text", text, &[]).map(|_| ());
    if let Err(error) = &result {
        report_error(error);
    }
    result
}

pub fn get_text() -> Option<String> {
    match bridge::request("clipboard_get_text", "", &[]) {
        Ok((text, _)) if text.is_empty() => None,
        Ok((text, _)) => Some(text),
        Err(error) => {
            report_error(&error);
            None
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/clipboard.rs"]
mod tests;
