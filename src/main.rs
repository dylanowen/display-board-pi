mod dot_matrix;
mod max7219;
mod sunrise_sunset_api;

use crate::dot_matrix::DotMatrix;

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
use std::thread::sleep;
use std::time::Duration;

lazy_static! {
    static ref LINE_STYLE: PrimitiveStyle<BinaryColor> =
        PrimitiveStyle::with_stroke(BinaryColor::On, 1);
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

    let mut matrix = DotMatrix::spi0(0x0)?;

    for h in 0..=12 {
        for m in 0..60 {
            matrix.clear()?;

            draw_up_arrow(5, 3, &mut matrix)?;
            draw_sun(5, 7, &mut matrix)?;

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
            matrix.flush()?;

            sleep(Duration::from_millis(50));

            Text::new(
                ":",
                Point::new(20, 5),
                MonoTextStyle::new(&FONT_4X6, BinaryColor::On),
            )
            .draw(&mut matrix)?;

            matrix.flush()?;

            sleep(Duration::from_millis(50));
        }
    }

    matrix.flush()?;

    // Circle::with_center(Point::new(5, 7), 5)
    //     .draw_styled(&PrimitiveStyle::with_fill(BinaryColor::On), &mut matrix);
    // let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    // Line::new(Point::new(5, 0), Point::new(5, 3)).draw_styled(&line_style, &mut matrix);
    // Line::new(Point::new(3, 1), Point::new(5, 3)).draw_styled(&line_style, &mut matrix);
    // Line::new(Point::new(7, 1), Point::new(5, 3)).draw_styled(&line_style, &mut matrix);

    //
    // matrix.write::<MonoFont>("abfg5", FONT_5X8);
    matrix.flush()?;

    sleep(Duration::from_secs(10));

    // let resp = reqwest::get(
    //     "https://api.sunrise-sunset.org/json?lat=40.743722&lng=-73.978020&formatted=0",
    // )
    // .await?
    // .json::<DaylightCollection>()
    // .await?;
    // println!("{:#?}", resp);

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

// fn draw_down_arrow<D>(x: i32, y: i32, target: &mut D)
// where
//     D: DrawTarget<Color = BinaryColor, Error = ()>,
// {
//     Line::new(Point::new(x, y - 3), Point::new(x, y)).draw_styled(&LINE_STYLE, target);
//     Line::new(Point::new(x - 2, y - 2), Point::new(x, y)).draw_styled(&LINE_STYLE, target);
//     Line::new(Point::new(x + 2, y - 2), Point::new(x, y)).draw_styled(&LINE_STYLE, target);
// }

fn draw_up_arrow<D, E>(x: i32, y: i32, target: &mut D) -> Result<(), E>
where
    D: DrawTarget<Color = BinaryColor, Error = E>,
{
    Line::new(Point::new(x, y - 3), Point::new(x, y)).draw_styled(&LINE_STYLE, target)?;
    Line::new(Point::new(x - 2, y - 1), Point::new(x, y - 3)).draw_styled(&LINE_STYLE, target)?;
    Line::new(Point::new(x + 2, y - 1), Point::new(x, y - 3)).draw_styled(&LINE_STYLE, target)?;

    Ok(())
}
