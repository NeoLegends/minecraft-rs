#![feature(async_await)]
#![allow(dead_code)]

use clap::{
    app_from_crate, crate_authors, crate_description, crate_name, crate_version,
    AppSettings, Arg,
};
use env_logger;
use log::error;
use std::path::Path;
use tokio;

const PORT_ARG: &str = "PORT";
const WORLD_ARG: &str = "WORLD";

#[tokio::main]
async fn main() {
    env_logger::init();

    let matches = app_from_crate!()
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::with_name(PORT_ARG)
                .long("port")
                .short("p")
                .help("The port to start the server on.")
                .default_value("25565")
                .required(true)
                .validator(|v| {
                    v.parse::<u16>().map(|_| ()).map_err(|_| {
                        format!("{} must be a valid port number", PORT_ARG)
                    })
                }),
        )
        .arg(
            Arg::with_name(WORLD_ARG)
                .long("world")
                .short("w")
                .help("The world to run.")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let port = matches.value_of(PORT_ARG).unwrap().parse().unwrap();
    let world = matches.value_of(WORLD_ARG).unwrap();

    let _ = minecraft::run(Path::new(world), port)
        .await
        .map_err(|e| error!("{:?}", e));
}
