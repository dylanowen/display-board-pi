mod dot_matrix;
mod max7219;
mod sunrise_sunset_api;

use crate::dot_matrix::DotMatrix;

use env_logger::Env;
use std::thread::sleep;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
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
    matrix.set_bit(0, 0, true);
    matrix.flush()?;
    sleep(Duration::from_millis(100));
    matrix.set_bit(1, 1, true);
    matrix.flush()?;
    sleep(Duration::from_millis(100));
    matrix.set_bit(2, 2, true);
    matrix.flush()?;
    sleep(Duration::from_millis(100));
    matrix.set_bit(3, 3, true);
    matrix.flush()?;
    sleep(Duration::from_millis(100));
    matrix.set_bit(4, 4, true);
    matrix.flush()?;
    sleep(Duration::from_millis(100));
    matrix.set_bit(5, 5, true);
    matrix.flush()?;
    sleep(Duration::from_millis(100));
    matrix.set_bit(6, 6, true);
    matrix.flush()?;
    sleep(Duration::from_millis(100));
    matrix.set_bit(7, 7, true);
    matrix.flush()?;
    sleep(Duration::from_millis(100));

    sleep(Duration::from_secs(10));

    // let resp = reqwest::get(
    //     "https://api.sunrise-sunset.org/json?lat=40.743722&lng=-73.978020&formatted=0",
    // )
    // .await?
    // .json::<Results>()
    // .await?;
    // println!("{:#?}", resp);

    // let device_info = DeviceInfo::new()?;
    //
    // println!("{:?}", device_info);
    //
    // // we have 4 segments chained together
    // let mut max = Max7219::spi0(4)?;
    //
    // // Scan Limit drives how many segments are shown, show all 7 of them
    // max.set_scan_limit(7);
    // // Decode mode doesn't make sense for our led-matrix disable it
    // max.set_decode_mode(DecodeMode::NoDecode);
    //
    // max.set_display_on(true);
    //
    // // turn off test_mode
    // max.set_display_test(false);
    //
    // max.write_all(Command::Digit0, 4);
    //
    // for intensity in 0..16 {
    //     sleep(Duration::from_millis(500));
    //     max.set_intensity(intensity);
    // }
    //
    // // let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 10_000_000, Mode::Mode0)?;
    // //
    // // spi.write(&[Command::ScanLimit as u8, 0b111])?;
    // //
    // // spi.write(&[Command::DisplayOn as u8, TRUE])?;
    // //
    // // spi.write(&[Command::DisplayTest as u8, FALSE])?;
    // //
    // sleep(Duration::from_secs(5));
    //
    // max.set_display_on(false);
    //
    // spi.write(&[Command::DisplayOn as u8, FALSE])?;

    // let args = Args::parse();
    //
    // setup_logging(&args)?;
    //
    // let runner = run_command(&args);
    // if args.timeout > 0 {
    //     timeout(Duration::from_secs(args.timeout), runner)
    //         .await
    //         .context("Operation timed out")
    //         .and_then(identity)?
    // } else {
    //     runner.await?
    // }

    // Ok(())
    println!("hi!");

    Ok(())
}
