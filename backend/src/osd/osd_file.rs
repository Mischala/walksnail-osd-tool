use std::{fs, path::PathBuf, time::Duration};

use derivative::Derivative;

use super::{error::OsdFileError, fc_firmware::FcFirmware};
use crate::osd::frame::Frame;

pub const HEADER_BYTES: usize = 40;
pub const FC_TYPE_BYTES: usize = 4;
pub const FRAME_BYTES: usize = 2124;

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct OsdFile {
    pub file_path: PathBuf,
    pub fc_firmware: FcFirmware,
    pub frame_count: u32,
    pub duration: Duration,
    pub version: Option<String>,
    #[derivative(Debug = "ignore")]
    pub frames: Vec<Frame>,
}

impl OsdFile {
    #[tracing::instrument(ret, err)]
    pub fn open(path: PathBuf) -> Result<Self, OsdFileError> {
        let mut bytes = fs::read(&path)?;
        let header_bytes = bytes.drain(0..HEADER_BYTES).collect::<Vec<u8>>();
        let fc_firmware = FcFirmware::try_from(&header_bytes[..FC_TYPE_BYTES])?;

        let frames = bytes
            .chunks(FRAME_BYTES)
            .map(|frame_bytes| frame_bytes.try_into().unwrap())
            .collect::<Vec<Frame>>();

        #[allow(clippy::cast_precision_loss)]
        let frame_interval = (frames.last().unwrap().time_millis - frames.first().unwrap().time_millis) as f32
            / (frames.len() - 1) as f32;

        let duration = Duration::from_millis(frames.last().unwrap().time_millis.into())
            + Duration::from_secs_f32(frame_interval / 1000.0);

        let mut osd_file = Self {
            file_path: path,
            fc_firmware,
            #[allow(clippy::cast_possible_truncation)]
            frame_count: frames.len() as u32,
            duration,
            version: None,
            frames,
        };

        if osd_file.fc_firmware == FcFirmware::Inav {
            osd_file.version = osd_file.detect_version();
        }

        Ok(osd_file)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn detect_version(&self) -> Option<String> {
        for frame in &self.frames {
            let mut text = String::new();
            for glyph in &frame.glyphs {
                let c = if glyph.index >= 0x20 && glyph.index <= 0x7E {
                    glyph.index as u8 as char
                } else {
                    ' '
                };
                text.push(c);
            }

            if let Some(pos) = text.find("INAV VERSION:") {
                let version_part = text[pos + 13..].trim_start();
                let version = version_part.split_whitespace().next().unwrap_or_default().to_string();
                if !version.is_empty() {
                    return Some(version);
                }
            }
        }
        None
    }

    pub fn save(&self) -> Result<(), OsdFileError> {
        let mut bytes = Vec::with_capacity(HEADER_BYTES + self.frames.len() * FRAME_BYTES);

        // Header: 4 bytes FC Type + 36 bytes padding
        let mut header = vec![0u8; HEADER_BYTES];
        let fc_bytes = self.fc_firmware.as_bytes();
        header[..fc_bytes.len()].copy_from_slice(fc_bytes);
        bytes.extend_from_slice(&header);

        // Frames
        for frame in &self.frames {
            bytes.extend_from_slice(&frame.as_bytes());
        }

        fs::write(&self.file_path, bytes)?;
        Ok(())
    }
}
