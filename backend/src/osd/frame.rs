use std::fmt::Debug;

use super::{
    error::OsdFileError,
    glyph::{Glyph, GridPosition},
    osd_file,
};

const TIMESTAMP_BYTES: usize = 4;
const BYTES_PER_GLYPH: usize = 2;
const GRID_WIDTH: usize = 53;
const _GRID_HEIGHT: usize = 20;

#[derive(Debug, Clone)]
pub struct Frame {
    pub time_millis: u32,
    pub glyphs: Vec<Glyph>,
}

impl TryFrom<&[u8]> for Frame {
    type Error = OsdFileError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let time_millis = u32::from_le_bytes(value[..TIMESTAMP_BYTES].try_into().unwrap());
        let glyphs = value[TIMESTAMP_BYTES..]
            .chunks(BYTES_PER_GLYPH)
            .enumerate()
            .filter_map(|(idx, glyph_bytes)| {
                let x = idx % GRID_WIDTH;
                let y = idx / GRID_WIDTH;
                let bytes = [glyph_bytes[0], glyph_bytes[1]];
                let index = u16::from_le_bytes(bytes);
                if index == 0x00 || index == 0x20 {
                    None
                } else {
                    let glyph = Glyph {
                        index,
                        grid_position: GridPosition {
                            #[allow(clippy::cast_possible_truncation)]
                            x: x as u32,
                            #[allow(clippy::cast_possible_truncation)]
                            y: y as u32,
                        },
                    };
                    Some(glyph)
                }
            })
            .collect();
        Ok(Self { time_millis, glyphs })
    }
}

impl Frame {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(osd_file::FRAME_BYTES);
        bytes.extend_from_slice(&self.time_millis.to_le_bytes());

        let mut grid = vec![0u16; GRID_WIDTH * _GRID_HEIGHT];
        for glyph in &self.glyphs {
            let idx = (glyph.grid_position.y as usize * GRID_WIDTH) + glyph.grid_position.x as usize;
            if idx < grid.len() {
                grid[idx] = glyph.index;
            }
        }

        for index in grid {
            bytes.extend_from_slice(&index.to_le_bytes());
        }

        bytes
    }
}
