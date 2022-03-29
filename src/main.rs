mod daylight;
mod dot_matrix;
mod max7219;
mod sunrise_sunset_api;

use crate::dot_matrix::DotMatrix;
use tokio::sync::mpsc;

use crate::daylight::Daylight;
use crate::sunrise_sunset_api::DaylightResponse;
use chrono::Utc;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::{FONT_4X6, FONT_5X7};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle, StyledDrawable};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use env_logger::Env;
use lazy_static::lazy_static;
use std::time::Duration;
use tokio::signal::ctrl_c;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

lazy_static! {
    static ref LINE_STYLE: PrimitiveStyle<BinaryColor> =
        PrimitiveStyle::with_stroke(BinaryColor::On, 1);
}

#[derive(Debug)]
enum Event {
    UpdateDisplay,
    Exit,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = Env::new().default_filter_or("info");
    #[cfg(not(feature = "max-simulator"))]
    env_logger::init_from_env(env);
    #[cfg(feature = "max-simulator")]
    {
        use env_logger::Builder;

        let filter = Builder::from_env(env).build().filter();
        tui_logger::init_logger(filter)?;
        tui_logger::set_default_level(filter);
    }

    let (tx, mut rx) = mpsc::channel(8);

    spawn_sigint_listener(&tx);
    spawn_display_updater(&tx);

    let mut matrix = DotMatrix::spi0(0x0)?;
    let mut daylight = Daylight::default();
    let mut show_colon = false;

    while let Some(event) = rx.recv().await {
        log::trace!("{event:?}");
        match event {
            Event::UpdateDisplay => {
                let now = Utc::now();
                daylight = daylight.update(now).await;

                matrix.clear()?;

                draw_sun(5, 7, &mut matrix)?;

                match daylight {
                    Daylight::Sunrise { .. } => draw_up_arrow(5, 3, &mut matrix)?,
                    Daylight::Sunset { .. } => draw_down_arrow(5, 3, &mut matrix)?,
                    Daylight::Unknown { .. } => (),
                }
                let (h, m) = daylight.until(now);

                Text::new(
                    &format!("{h:02}"),
                    Point::new(11, 6),
                    MonoTextStyle::new(&FONT_5X7, BinaryColor::On),
                )
                .draw(&mut matrix)?;
                Text::new(
                    &format!("{m:02}"),
                    Point::new(23, 6),
                    MonoTextStyle::new(&FONT_5X7, BinaryColor::On),
                )
                .draw(&mut matrix)?;

                show_colon = !show_colon;
                if show_colon {
                    Text::new(
                        ":",
                        Point::new(20, 5),
                        MonoTextStyle::new(&FONT_4X6, BinaryColor::On),
                    )
                    .draw(&mut matrix)?;
                }

                matrix.flush()?;
            }
            Event::Exit => break,
        }
    }

    Ok(())
}

fn draw_sun<D, E>(x: i32, y: i32, target: &mut D) -> Result<(), E>
where
    D: DrawTarget<Color = BinaryColor, Error = E>,
{
    let diameter = 5;
    Circle::with_center(Point::new(x, y), diameter)
        .draw_styled(&PrimitiveStyle::with_fill(BinaryColor::On), target)?;

    Line::new(Point::new(x - 5, y), Point::new(x - 4, y)).draw_styled(&LINE_STYLE, target)?;
    Line::new(Point::new(x + 5, y), Point::new(x + 4, y)).draw_styled(&LINE_STYLE, target)?;
    Line::new(Point::new(x - 4, y - 4), Point::new(x - 3, y - 3))
        .draw_styled(&LINE_STYLE, target)?;
    Line::new(Point::new(x + 4, y - 4), Point::new(x + 3, y - 3))
        .draw_styled(&LINE_STYLE, target)?;

    Ok(())
}

fn draw_down_arrow<D, E>(x: i32, y: i32, target: &mut D) -> Result<(), E>
where
    D: DrawTarget<Color = BinaryColor, Error = E>,
{
    Line::new(Point::new(x, y - 3), Point::new(x, y)).draw_styled(&LINE_STYLE, target)?;
    Line::new(Point::new(x - 2, y - 2), Point::new(x, y)).draw_styled(&LINE_STYLE, target)?;
    Line::new(Point::new(x + 2, y - 2), Point::new(x, y)).draw_styled(&LINE_STYLE, target)?;

    Ok(())
}

fn draw_up_arrow<D, E>(x: i32, y: i32, target: &mut D) -> Result<(), E>
where
    D: DrawTarget<Color = BinaryColor, Error = E>,
{
    Line::new(Point::new(x, y - 3), Point::new(x, y)).draw_styled(&LINE_STYLE, target)?;
    Line::new(Point::new(x - 2, y - 1), Point::new(x, y - 3)).draw_styled(&LINE_STYLE, target)?;
    Line::new(Point::new(x + 2, y - 1), Point::new(x, y - 3)).draw_styled(&LINE_STYLE, target)?;

    Ok(())
}

fn spawn_sigint_listener(tx: &Sender<Event>) {
    let tx = tx.clone();
    tokio::spawn(async move {
        if let Err(error) = ctrl_c().await {
            log::error!("{error}");
        }
        log::info!("Exiting");
        send_log(Event::Exit, &tx).await;
    });
}

fn spawn_display_updater(tx: &Sender<Event>) {
    let tx = tx.clone();
    tokio::spawn(async move {
        loop {
            send_log(Event::UpdateDisplay, &tx).await;
            // Update our display ever 1s
            sleep(Duration::from_secs(1)).await;
        }
    });
}

async fn send_log<T>(value: T, tx: &Sender<T>) {
    if let Err(error) = tx.send(value).await {
        log::error!("Error Sending: {error}");
    }
}
