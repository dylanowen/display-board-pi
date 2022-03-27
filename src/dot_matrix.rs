use crate::max7219::{
    DecodeMode, Intensity, Max7219, COMMAND_BITS, DATA_BITS, DIGITS, INSTRUCTION_BITS,
    INSTRUCTION_BYTES,
};

use bitvec::order::Msb0;
use bitvec::prelude::*;

use bitvec::bitarr;

pub const CHAINED_SEGMENTS: usize = 4;

pub const DISPLAY_WIDTH: usize = DATA_BITS;
pub const DISPLAY_HEIGHT: usize = DIGITS.len();
//
pub const ROW_LENGTH: usize = CHAINED_SEGMENTS * DISPLAY_WIDTH;

/// Display Coordinates
/// 0,0 →  x
/// ↓
/// y
pub struct DotMatrix {
    pub max: Max7219,
    /// Prepare our buffer to hold an entire "frame"
    display_buffer: BitArr!(for CHAINED_SEGMENTS * DISPLAY_HEIGHT * INSTRUCTION_BITS, in u8, Msb0),
    intensity: Intensity,
}

impl DotMatrix {
    pub fn spi0(intensity: Intensity) -> anyhow::Result<DotMatrix> {
        let mut max = Max7219::spi0(4)?;

        // Scan Limit drives how many segments are shown, show all 7 of them
        max.set_scan_limit(7)?;
        // Decode mode doesn't make sense for our led-matrix, disable it
        max.set_decode_mode(DecodeMode::NoDecode)?;
        // turn off test_mode
        max.set_display_test(false)?;

        max.set_display_on(true)?;

        let mut display_buffer =
            bitarr![u8, Msb0; 0b0; CHAINED_SEGMENTS * DISPLAY_HEIGHT * INSTRUCTION_BITS];

        for y in 0..DISPLAY_HEIGHT {
            for x_row in 0..CHAINED_SEGMENTS {
                display_buffer.data[((CHAINED_SEGMENTS * y) + x_row) * INSTRUCTION_BYTES] =
                    DIGITS[y] as u8;
            }
        }

        let mut matrix = DotMatrix {
            max,
            display_buffer,
            intensity: 0x0,
        };
        matrix.set_intensity(intensity)?;

        Ok(matrix)
    }

    pub fn set_intensity(&mut self, intensity: Intensity) -> anyhow::Result<()> {
        self.intensity = intensity;
        self.max.set_intensity(intensity)
    }

    pub fn clear(&mut self) -> anyhow::Result<()> {
        for y in 0..DISPLAY_HEIGHT {
            for x_row in 0..CHAINED_SEGMENTS {
                self.set_byte(x_row, y, 0b00000000);
            }
            self.flush_row(y)?;
        }

        Ok(())
    }

    pub fn all_on(&mut self) -> anyhow::Result<()> {
        for y in 0..DIGITS.len() {
            for x_row in 0..CHAINED_SEGMENTS {
                self.set_byte(x_row, y, 0b11111111);
            }
            self.flush_row(y)?;
        }

        Ok(())
    }

    pub fn get_bit(&self, x: usize, y: usize) -> bool {
        self.display_buffer[offset(x, y)]
    }

    pub fn set_bit(&mut self, x: usize, y: usize, value: bool) {
        self.display_buffer.set(offset(x, y), value)
    }

    pub fn get_byte(&self, x_row: usize, y: usize) -> u8 {
        self.display_buffer.data[((y * CHAINED_SEGMENTS) + x_row) * 2 + 1]
    }

    pub fn set_byte(&mut self, x_row: usize, y: usize, data: u8) {
        self.display_buffer.data[((y * CHAINED_SEGMENTS) + x_row) * 2 + 1] = data
    }

    pub fn flush(&mut self) -> anyhow::Result<()> {
        for y in 0..DIGITS.len() {
            self.flush_row(y)?;
        }
        Ok(())
    }

    fn flush_row(&mut self, y: usize) -> anyhow::Result<()> {
        let start = y * CHAINED_SEGMENTS * 2;
        let end = start + (CHAINED_SEGMENTS * 2);
        let row_data = &self.display_buffer.data[start..end];
        self.max.write(row_data)?;
        Ok(())
    }
}

impl Drop for DotMatrix {
    fn drop(&mut self) {
        if let Err(error) = self.max.set_display_on(false) {
            log::error!("Failed to shutdown display: {error:?}")
        }
    }
}

fn offset(x: usize, y: usize) -> usize {
    let y_offset = y * (CHAINED_SEGMENTS * INSTRUCTION_BITS);
    let x_segments = x / DATA_BITS;

    y_offset + (x_segments * INSTRUCTION_BITS) + COMMAND_BITS + (x % DATA_BITS)
}
