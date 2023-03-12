use std::error::Error;
use std::path::PathBuf;

use clap::{arg, command, value_parser};
use tokio::signal;

use energy_monitor::actor::datalogger::DataLoggerActor;
use energy_monitor::actor::hmi::HmiActor;
use energy_monitor::actor::linky::LinkyActor;
use energy_monitor::actor::rpict::RpictActor;
use energy_monitor::settings::Settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = command!()
        .arg(arg!(-c --config <FILE> "Sets a custom config file").required(false).value_parser(value_parser!(PathBuf)))
        .get_matches();
    let settings = Settings::new(matches.get_one::<PathBuf>("config")).expect("Can't load settings");
    env_logger::Builder::new().parse_filters(&settings.log_level).init();
    log::debug!("{:?}", settings);

    let rpict = RpictActor::create(&settings.serial.rpict);
    let linky = LinkyActor::create(&settings.serial.linky);
    let datalogger = DataLoggerActor::create(&settings.influxdb, &rpict, &linky)?;
    let hmi = HmiActor::create(&settings.hmi, &rpict, &linky, &datalogger)?;

    let _ = signal::ctrl_c().await;
    hmi.shutdown().await;
    Ok(())
}
