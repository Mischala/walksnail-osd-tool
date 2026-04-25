use std::{path::Path, process::Command, time::Duration};

use regex::Regex;

use super::{
    error::OsdFileError,
    fc_firmware::FcFirmware,
    frame::Frame,
    glyph::{Glyph, GridPosition},
    osd_file::OsdFile,
};

const GRID_WIDTH: usize = 53;
const GRID_HEIGHT: usize = 20;

/// Extract SEI User Data entries from a video file using ffmpeg showinfo filter.
/// Returns a list of (`pts_seconds`, `hex_string`) tuples.
fn extract_sei_data(ffmpeg_path: &Path, video_path: &Path, max_duration: Option<Duration>) -> Vec<(f64, String)> {
    let mut command = Command::new(ffmpeg_path);

    let duration_str = max_duration.map(|t| format!("{:.3}", t.as_secs_f64()));
    if let Some(t) = &duration_str {
        command.arg("-t").arg(t);
    }

    command.args([
        "-hwaccel",
        "auto",
        "-i",
        video_path.to_str().unwrap_or(""),
        "-vf",
        "showinfo=checksum=false",
        "-f",
        "null",
        "-",
    ]);
    command.stdout(std::process::Stdio::null());
    command.stderr(std::process::Stdio::piped());

    #[cfg(target_os = "windows")]
    std::os::windows::process::CommandExt::creation_flags(&mut command, crate::util::CREATE_NO_WINDOW);

    let output = match command.output() {
        Ok(o) => o,
        Err(e) => {
            tracing::error!("Failed to run ffmpeg showinfo: {}", e);
            return Vec::new();
        }
    };

    let stderr = String::from_utf8_lossy(&output.stderr);

    let pattern = Regex::new(r"(?s)pts_time:([\d.]+).*?User Data=([0-9a-fA-F\s]+)").expect("Invalid regex");

    let entries: Vec<_> = pattern
        .captures_iter(&stderr)
        .filter_map(|cap| {
            let pts: f64 = cap[1].parse().ok()?;
            let hex = cap[2].to_string();
            tracing::debug!(
                "Captured SEI hex (first 50 chars): {}",
                &hex.chars().take(50).collect::<String>()
            );
            Some((pts, hex))
        })
        .collect();

    tracing::info!("Found {} SEI User Data entries in stderr", entries.len());
    entries
}

/// Parse an MSP `DisplayPort` payload from hex string.
/// Returns a list of (row, col, `glyph_index`) tuples.
#[allow(clippy::cast_possible_truncation)]
fn parse_msp_payload(hex_string: &str) -> Option<Vec<(u8, u8, u16)>> {
    // 1. Clean the string: remove address prefixes like "00000010:" and colons
    // showinfo often formats SEI data with address prefixes.
    let mut cleaned = String::with_capacity(hex_string.len());
    for line in hex_string.lines() {
        let line = line.trim();
        // If line starts with an address like "00000000: ", skip the address part
        if let Some(pos) = line.find(": ") {
            cleaned.push_str(&line[pos + 2..]);
        } else {
            cleaned.push_str(line);
        }
    }

    // 2. Remove all non-hex characters (except spaces which hex::decode handles poorly if not removed)
    let clean_hex: String = cleaned.chars().filter(char::is_ascii_hexdigit).collect();

    // Convert to bytes
    let raw_bytes = match hex::decode(&clean_hex) {
        Ok(b) => b,
        Err(e) => {
            tracing::debug!(
                "Hex decode failed for string: {}... error: {}",
                &clean_hex.chars().take(20).collect::<String>(),
                e
            );
            return None;
        }
    };

    // Structural removal: Remove every 3rd byte (the padding byte)
    // Artlynk SEI format often packs 2 bytes of data and 1 byte of filler (0xff)
    let mut data = Vec::with_capacity(raw_bytes.len() * 2 / 3);
    for (i, &b) in raw_bytes.iter().enumerate() {
        if (i + 1) % 3 != 0 {
            data.push(b);
        }
    }

    if data.len() < 10 {
        return None;
    }

    // Find all occurrences of the command prefix b6 03 (MSP DisplayPort Write String)
    let mut command_offsets = Vec::new();
    for i in 0..(data.len().saturating_sub(1)) {
        if data[i] == 0xb6 && data[i + 1] == 0x03 {
            command_offsets.push(i);
        }
    }

    if command_offsets.is_empty() {
        return None;
    }

    let mut active_glyphs = Vec::new();

    for (i, &start_offset) in command_offsets.iter().enumerate() {
        // The header is b6 03 ROW COL ATTR (5 bytes)
        if start_offset + 5 > data.len() {
            continue;
        }

        let row = data[start_offset + 2];
        let col = data[start_offset + 3];
        let attribute = data[start_offset + 4];

        // Glyph data starts after the 5-byte header
        let glyph_start = start_offset + 5;
        
        // Glyph data ends before the next command's length byte
        // (The length byte is typically one byte before the next b6 03)
        let glyph_end = if i + 1 < command_offsets.len() {
            command_offsets[i + 1].saturating_sub(1)
        } else {
            data.len()
        };

        if glyph_start < glyph_end {
            let num_glyphs = glyph_end - glyph_start;
            for j in 0..num_glyphs {
                if glyph_start + j >= data.len() {
                    break;
                }
                let glyph_byte = u16::from(data[glyph_start + j]);
                let character = ((u16::from(attribute) & 0x03) << 8) | glyph_byte;
                active_glyphs.push((row, col.saturating_add(j as u8), character));
            }
        }
    }

    if active_glyphs.is_empty() {
        None
    } else {
        Some(active_glyphs)
    }
}

#[tracing::instrument(ret, err)]
#[allow(clippy::cast_precision_loss)]
pub fn extract_osd_from_video(ffmpeg_path: &Path, video_path: &Path) -> Result<Option<OsdFile>, OsdFileError> {
    // 0. Check if an OSD file already exists
    let osd_path = video_path.with_extension("osd");
    if osd_path.exists() {
        tracing::info!("Found existing OSD file {:?}, skipping scan.", osd_path);
        return Ok(Some(OsdFile::open(osd_path)?));
    }

    tracing::info!("Attempting Artlynk OSD extraction from {:?}", video_path);

    // 1. Quick check: scan first 2 seconds to see if SEI data exists
    let quick_entries = extract_sei_data(ffmpeg_path, video_path, Some(Duration::from_secs(2)));
    if quick_entries.is_empty() {
        tracing::info!("No SEI User Data found in first 2 seconds, skipping full scan.");
        return Ok(None);
    }

    // 2. Full scan: if SEI data was found, extract everything
    let entries = extract_sei_data(ffmpeg_path, video_path, None);
    if entries.is_empty() {
        tracing::info!("No SEI User Data found in video during full scan");
        return Ok(None);
    }

    let mut frames = Vec::new();

    for (pts, hex_line) in &entries {
        let Some(glyphs_raw) = parse_msp_payload(hex_line) else {
            continue;
        };

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let ts_ms = (*pts * 1000.0) as u32;

        let glyphs: Vec<Glyph> = glyphs_raw
            .into_iter()
            .filter(|&(row, col, idx)| {
                (col as usize) < GRID_WIDTH && (row as usize) < GRID_HEIGHT && idx != 0x00 && idx != 0x20
            })
            .map(|(row, col, idx)| Glyph {
                index: idx,
                grid_position: GridPosition {
                    x: u32::from(col),
                    y: u32::from(row),
                },
            })
            .collect();

        if !glyphs.is_empty() {
            frames.push(Frame {
                time_millis: ts_ms,
                glyphs,
            });
        }
    }

    if frames.is_empty() {
        tracing::info!("SEI data found but no valid OSD frames parsed");
        return Ok(None);
    }

    #[allow(clippy::cast_possible_truncation)]
    let frame_count = u32::try_from(frames.len()).unwrap_or(u32::MAX);

    let frame_interval = if frames.len() > 1 {
        (frames.last().unwrap().time_millis - frames.first().unwrap().time_millis) as f32 / (frames.len() - 1) as f32
    } else {
        33.0 // ~30fps default
    };

    let duration = Duration::from_millis(frames.last().unwrap().time_millis.into())
        + Duration::from_secs_f32(frame_interval / 1000.0);

    tracing::info!("Extracted {} OSD frames from Artlynk SEI data", frame_count);

    let osd_file = OsdFile {
        file_path: osd_path,
        fc_firmware: FcFirmware::Betaflight,
        frame_count,
        duration,
        version: None,
        frames,
    };

    // Save the extracted data to the .osd file
    if let Err(e) = osd_file.save() {
        tracing::error!("Failed to save extracted OSD data to {:?}: {}", osd_file.file_path, e);
    } else {
        tracing::info!("Saved extracted OSD data to {:?}", osd_file.file_path);
    }

    Ok(Some(osd_file))
}
