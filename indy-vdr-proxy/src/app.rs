extern crate clap;
use clap::{Arg, Command};

pub struct Config {
    pub genesis: String,
    #[cfg(unix)]
    pub socket: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub init_refresh: bool,
    pub interval_refresh: u32,
}

pub fn load_config() -> Result<Config, String> {
    #[allow(unused_mut)]
    let mut app = Command::new("indy-vdr-proxy")
        .version("0.1.0")
        .about("Proxy requests to a Hyperledger Indy-Node ledger")
        .arg(
            Arg::new("genesis")
                .short('g')
                .long("genesis")
                .takes_value(true)
                .value_name("GENESIS")
                .help("Path to the ledger genesis transactions")
        )
        .arg(
            Arg::new("host")
                .short('h')
                .long("host")
                .takes_value(true)
                .value_name("HOST")
                .default_value("0.0.0.0")
                .help("Set the local address to listen on")
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .takes_value(true)
                .value_name("PORT")
                .help("Sets the local port to listen on")
        )
        .arg(
            Arg::new("no-refresh")
                .help("Disable initial validator node refresh"),
        )
        .arg(
            Arg::new("refresh-interval")
                .short('r')
                .long("refresh-interval")
                .takes_value(true)
                .value_name("INTERVAL")
                .help("Set the interval in minutes between validator node refresh attempts (0 to disable refresh, default 120)"),
        );

    #[cfg(unix)]
    {
        app = app.arg(
            Arg::new("socket")
                .short('s')
                .long("socket")
                .takes_value(true)
                .value_name("SOCKET")
                .help("Sets the UNIX socket path listen on"),
        );
    }

    let matches = app.get_matches();

    let genesis = matches
        .value_of("genesis")
        .unwrap_or("genesis.txn")
        .to_owned();

    if matches.occurrences_of("socket") > 0 {
        if matches.occurrences_of("host") > 0 {
            return Err("Cannot specify both host and socket".to_owned());
        }
    } else {
        if matches.occurrences_of("port") == 0 {
            return Err("Port number or socket must be specified".to_owned());
        }
    }

    #[cfg(unix)]
    let socket = matches.value_of("socket").map(str::to_owned);

    let host = matches.value_of("host").map(str::to_owned);
    let port = if let Some(port) = matches.value_of("port") {
        Some(port.parse::<u16>().map_err(|_| "Invalid port number")?)
    } else {
        None
    };
    let init_refresh = !matches.is_present("no-refresh");
    let interval_refresh = matches
        .value_of("refresh-interval")
        .map(|ival| ival.parse::<u32>().map_err(|_| "Invalid refresh interval"))
        .transpose()?
        .unwrap_or(120);

    Ok(Config {
        genesis,
        #[cfg(unix)]
        socket,
        host,
        port,
        init_refresh,
        interval_refresh,
    })
}
