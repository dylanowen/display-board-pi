use std::mem::size_of;

#[allow(dead_code)]
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Command {
    Noop = 0x0,
    Digit0 = 0x1,
    Digit1 = 0x2,
    Digit2 = 0x3,
    Digit3 = 0x4,
    Digit4 = 0x5,
    Digit5 = 0x6,
    Digit6 = 0x7,
    Digit7 = 0x8,
    DecodeMode = 0x9,
    Intensity = 0xA,
    ScanLimit = 0xB,
    DisplayOn = 0xC,
    DisplayTest = 0xF,
}

pub const COMMAND_BYTES: usize = size_of::<Command>();
pub const COMMAND_BITS: usize = COMMAND_BYTES * u8::BITS as usize;

pub const DIGITS: [Command; 8] = [
    Command::Digit0,
    Command::Digit1,
    Command::Digit2,
    Command::Digit3,
    Command::Digit4,
    Command::Digit5,
    Command::Digit6,
    Command::Digit7,
];

pub type Intensity = u8;
pub type ScanLimit = u8;

#[allow(dead_code)]
#[repr(u8)]
pub enum DecodeMode {
    NoDecode = 0b00000000,
    Decode0 = 0b00000001,
    Decode3_0 = 0b00001111,
    DecodeAll = 0b11111111,
}

pub const DATA_BYTES: usize = 1;
pub const DATA_BITS: usize = DATA_BYTES * u8::BITS as usize;

pub const INSTRUCTION_BYTES: usize = COMMAND_BYTES + DATA_BYTES;
pub const INSTRUCTION_BITS: usize = INSTRUCTION_BYTES * u8::BITS as usize;

use max_impl::Max7219Impl;

pub struct Max7219 {
    chained_segments: usize,
    max_impl: Max7219Impl,
}

impl Max7219 {
    pub fn spi0(chained_segments: usize) -> anyhow::Result<Max7219> {
        Ok(Max7219 {
            chained_segments,
            max_impl: Max7219Impl::spi0(chained_segments)?,
        })
    }

    pub fn set_decode_mode(&mut self, decode_mode: DecodeMode) -> anyhow::Result<()> {
        self.write_all(Command::DecodeMode, decode_mode as u8)
    }

    pub fn set_intensity(&mut self, intensity: Intensity) -> anyhow::Result<()> {
        self.write_all(Command::Intensity, intensity)
    }

    pub fn set_scan_limit(&mut self, display_digits: ScanLimit) -> anyhow::Result<()> {
        self.write_all(Command::ScanLimit, display_digits)
    }

    pub fn set_display_on(&mut self, display_on: bool) -> anyhow::Result<()> {
        self.write_all(Command::DisplayOn, display_on as u8)
    }

    pub fn set_display_test(&mut self, display_on: bool) -> anyhow::Result<()> {
        self.write_all(Command::DisplayTest, display_on as u8)
    }

    // Write our command and data to ALL the chained Maxes
    pub fn write_all(&mut self, command: Command, data: u8) -> anyhow::Result<()> {
        let mut buffer = vec![0b0; self.chained_segments * 2];
        for display in 0..self.chained_segments {
            buffer[display * 2] = command as u8;
            buffer[display * 2 + 1] = data;
        }

        self.write(&buffer)
    }

    pub fn write(&mut self, data: &[u8]) -> anyhow::Result<()> {
        log::trace!("Writing: {data:02X?}");

        self.max_impl.write(data)
    }
}

#[cfg(feature = "max-physical")]
mod max_impl {
    use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

    pub(crate) struct Max7219Impl {
        spi: Spi,
    }

    impl Max7219Impl {
        pub(super) fn spi0(_chained_segments: usize) -> anyhow::Result<Max7219Impl> {
            let channel = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 10_000_000, Mode::Mode0)?;

            log::info!("Connected to MAX7219");
            Ok(Max7219Impl { spi: channel })
        }

        pub fn write(&mut self, data: &[u8]) -> anyhow::Result<()> {
            self.spi.write(data)?;

            Ok(())
        }
    }
}

#[cfg(feature = "max-simulator")]
mod max_impl {
    use crate::dot_matrix::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
    use crate::max7219::Command;
    use bitvec::order::Msb0;
    use bitvec::BitArr;
    use std::io;
    use std::io::Stdout;
    use tui::backend::CrosstermBackend;
    use tui::buffer::Buffer;
    use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
    use tui::style::{Color, Style};
    use tui::widgets::{Block, BorderType, Borders, Widget};
    use tui::Terminal;
    use tui_logger::TuiLoggerWidget;

    pub(crate) struct Max7219Impl {
        terminal: Terminal<CrosstermBackend<Stdout>>,
        panels: Vec<Panel>,
    }

    impl Max7219Impl {
        pub(super) fn spi0(chained_segments: usize) -> anyhow::Result<Max7219Impl> {
            let stdout = io::stdout();
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;
            terminal.clear()?;

            let mut panels = Vec::with_capacity(chained_segments);
            panels.resize_with(chained_segments, Panel::default);

            log::info!("Connected to MAX7219 Simulator");
            Ok(Max7219Impl { terminal, panels })
        }

        pub fn write(&mut self, data: &[u8]) -> anyhow::Result<()> {
            for i in (0..data.len()).step_by(2) {
                let command = data[i];
                let data = data[i + 1];
                let panel_i = i / 2;

                if let Some(panel) = self.panels.get_mut(panel_i) {
                    if command == Command::DisplayOn as u8 {
                        panel.on = data & 0x1 == 0x1;
                        // log::trace!("Panel {panel_i} on=={}", panel.on);
                    } else if command >= Command::Digit0 as u8 && command <= Command::Digit7 as u8 {
                        let row = command - Command::Digit0 as u8;
                        // log::trace!("Panel {panel_i} Row {row} {data:08b}");
                        panel.data.data[row as usize] = data;
                    }
                }
            }

            let rendered_panels = self.panels.clone();
            self.terminal.draw(|frame| {
                let vertical_layouts = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![
                        // make sure we have room for our borders
                        Constraint::Length(2 + DISPLAY_HEIGHT as u16),
                        Constraint::Min(10),
                    ])
                    .split(frame.size());

                let max_block = Block::default()
                    .borders(Borders::ALL)
                    .title("MAX7219")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded);
                let border_area = max_block.inner(vertical_layouts[0]);
                frame.render_widget(max_block, vertical_layouts[0]);

                let panel_layouts = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![
                        Constraint::Length(DISPLAY_WIDTH as u16);
                        rendered_panels.len()
                    ])
                    .split(border_area);

                for (i, panel) in rendered_panels.into_iter().enumerate() {
                    let layout = panel_layouts[i];
                    frame.render_widget(panel, layout);
                }

                let logger = TuiLoggerWidget::default()
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Logs")
                            .title_alignment(Alignment::Center)
                            .border_type(BorderType::Rounded),
                    )
                    .style_error(Style::default().fg(Color::Red))
                    .style_debug(Style::default().fg(Color::Green))
                    .style_warn(Style::default().fg(Color::Yellow))
                    .style_trace(Style::default().fg(Color::Magenta))
                    .style_info(Style::default().fg(Color::Cyan))
                    .output_file(false);
                frame.render_widget(logger, vertical_layouts[1]);
            })?;

            Ok(())
        }
    }

    #[derive(Default, Clone)]
    struct Panel {
        on: bool,
        data: BitArr!(for DISPLAY_WIDTH * DISPLAY_HEIGHT, in u8, Msb0),
    }

    impl Widget for Panel {
        fn render(self, area: Rect, buf: &mut Buffer) {
            if self.on {
                for y in 0..DISPLAY_HEIGHT.min(area.height as usize) {
                    for x in 0..DISPLAY_WIDTH.min(area.width as usize) {
                        let i = (y * DISPLAY_WIDTH) + x;
                        let screen_x = x as u16 + area.x;
                        let screen_y = y as u16 + area.y;

                        if self.data[i] {
                            buf.set_string(screen_x, screen_y, "●", Style::default())
                        }
                    }
                }
            }
        }
    }
}
