use std::{path::PathBuf, time::Duration};

use ffprobe::FfProbe;

use super::error::VideoInfoError;

#[derive(Debug)]
pub struct VideoInfo {
    pub width: u32,
    pub height: u32,
    pub frame_rate: f32,
    pub bitrate: u32,
    pub duration: Duration,
    pub total_frames: u32,
}

impl VideoInfo {
    #[tracing::instrument(ret)]
    pub fn get(file_path: &PathBuf, ffprobe_path: &PathBuf) -> Result<Self, VideoInfoError> {
        let info = ffprobe::ffprobe(file_path, ffprobe_path.clone())?;
        info.try_into()
    }
}

impl TryFrom<FfProbe> for VideoInfo {
    type Error = VideoInfoError;

    fn try_from(value: FfProbe) -> Result<Self, Self::Error> {
        let stream = value.streams.first().ok_or(VideoInfoError::NoStream)?;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let width = stream.width.ok_or(VideoInfoError::NoFrameWidth)? as u32;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let height = stream.height.ok_or(VideoInfoError::NoFrameHeight)? as u32;
        let frame_rate = {
            let frame_rate_string = &stream.avg_frame_rate;
            let mut split = frame_rate_string.split('/');
            let num = split
                .next()
                .and_then(|num| num.parse::<f32>().ok())
                .ok_or(VideoInfoError::NoFrameRate)?;
            let den = split
                .next()
                .and_then(|num| num.parse::<f32>().ok())
                .ok_or(VideoInfoError::NoFrameRate)?;
            let fps = num / den;
            round_to_standard_fps(fps)
        };
        let bitrate = stream
            .bit_rate
            .as_ref()
            .and_then(|b| b.parse::<u32>().ok())
            .ok_or(VideoInfoError::NoBitrate)?;

        let duration = Duration::from_secs_f32(
            stream
                .duration
                .as_ref()
                .and_then(|s| s.parse::<f32>().ok())
                .ok_or(VideoInfoError::NoDuration)?,
        );

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let total_frames = (frame_rate * duration.as_secs_f32()) as u32;

        Ok(Self {
            width,
            height,
            frame_rate,
            bitrate,
            duration,
            total_frames,
        })
    }
}

fn round_to_standard_fps(fps: f32) -> f32 {
    let standard_rates = [
        23.976, 24.0, 25.0, 29.97, 30.0, 48.0, 50.0, 59.94, 60.0, 90.0, 100.0, 120.0,
    ];
    for &rate in &standard_rates {
        if (fps - rate).abs() < (rate * 0.05) {
            return rate;
        }
    }
    fps
}
