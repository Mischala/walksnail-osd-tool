use std::{fmt::Display, path::PathBuf, process::Command};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Codec {
    H264,
    H265,
    AV1,
}

impl Display for Codec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::H264 => write!(f, "H.264"),
            Self::H265 => write!(f, "H.265"),
            Self::AV1 => write!(f, "AV1"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Encoder {
    pub name: String,
    pub codec: Codec,
    pub hardware: bool,
    pub detected: bool,
    pub extra_args: Vec<String>,
}

impl Encoder {
    fn new(name: &str, codec: Codec, hardware: bool) -> Self {
        Self::new_with_extra_args(name, codec, hardware, &[])
    }

    fn new_with_extra_args(name: &str, codec: Codec, hardware: bool, extra_args: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            codec,
            hardware,
            detected: false,
            extra_args: extra_args.iter().map(|&s| s.to_string()).collect(),
        }
    }

    #[tracing::instrument(ret)]
    pub fn get_available_encoders(ffmpeg_path: &PathBuf) -> Vec<Self> {
        #[rustfmt::skip]
        let mut all_encoders = [
            Self::new("libx264", Codec::H264, false),
            Self::new("libx265", Codec::H265, false),

            #[cfg(target_os = "windows")]
            Self::new("h264_amf", Codec::H264, true),

            #[cfg(any(target_os = "windows", target_os = "linux"))]
            Self::new("h264_nvenc", Codec::H264, true),

            #[cfg(any(target_os = "windows", target_os = "linux"))]
            Self::new("h264_qsv", Codec::H264, true),

            #[cfg(target_os = "linux")]
            Self::new("h264_vaapi", Codec::H264, true),

            #[cfg(target_os = "linux")]
            Self::new("h264_v4l2m2m", Codec::H264, true),

            #[cfg(target_os = "macos")]
            Self::new("h264_videotoolbox", Codec::H264, true),

            #[cfg(target_os = "windows")]
            Self::new("hevc_amf", Codec::H265, true),

            #[cfg(any(target_os = "windows", target_os = "linux"))]
            Self::new("hevc_nvenc", Codec::H265, true),

            #[cfg(any(target_os = "windows", target_os = "linux"))]
            Self::new("hevc_qsv", Codec::H265, true),

            #[cfg(target_os = "linux")]
            Self::new("hevc_vaapi", Codec::H265, true),

            #[cfg(target_os = "linux")]
            Self::new("hevc_v4l2m2m", Codec::H265, true),

            #[cfg(target_os = "macos")]
            Self::new_with_extra_args(
                "hevc_videotoolbox", Codec::H265, true,
                &["-tag:v", "hvc1"] // Apple QuickTime player on Mac only supports hvc1
            ),

            Self::new("libaom-av1", Codec::AV1, false),
            Self::new("libsvtav1", Codec::AV1, false),
            Self::new("librav1e", Codec::AV1, false),

            #[cfg(target_os = "windows")]
            Self::new("av1_amf", Codec::AV1, true),

            #[cfg(any(target_os = "windows", target_os = "linux"))]
            Self::new("av1_nvenc", Codec::AV1, true),

            #[cfg(any(target_os = "windows", target_os = "linux"))]
            Self::new("av1_qsv", Codec::AV1, true),

            #[cfg(target_os = "linux")]
            Self::new("av1_vaapi", Codec::AV1, true),

            #[cfg(target_os = "linux")]
            Self::new("av1_v4l2m2m", Codec::AV1, true),
        ];

        all_encoders
            .par_iter_mut()
            .map(|encoder| {
                encoder.detected = Self::ffmpeg_encoder_available(encoder, ffmpeg_path);
                encoder.clone()
            })
            .collect()
    }

    fn ffmpeg_encoder_available(encoder: &Self, ffmpeg_path: &PathBuf) -> bool {
        let mut command = Command::new(ffmpeg_path);

        command
            .args([
                "-hide_banner",
                "-f",
                "lavfi",
                "-i",
                "nullsrc",
                "-c:v",
                &encoder.name,
                "-frames:v",
                "1",
                "-f",
                "null",
                "-",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        #[cfg(target_os = "windows")]
        std::os::windows::process::CommandExt::creation_flags(&mut command, crate::util::CREATE_NO_WINDOW);

        command.status().is_ok_and(|status| status.success())
    }
}

impl Display for Encoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} — {} — {}",
            self.name,
            self.codec,
            if self.hardware { "hardware" } else { "software" }
        )
    }
}
