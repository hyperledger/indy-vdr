extern crate clap;
use clap::{Arg, ArgAction, Command};

pub struct Config {
    pub genesis: Option<String>,
    pub namespace: String,
    #[cfg(unix)]
    pub socket: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub init_refresh: bool,
    pub interval_refresh: u32,
    pub is_multiple: bool,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
}

pub fn load_config() -> Result<Config, String> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    #[allow(unused_mut)]
    let mut app = Command::new("indy-vdr-proxy")
        .version(VERSION)
        .about("Proxy requests to a Hyperledger Indy-Node ledger")
        .arg(
            Arg::new("genesis")
                .short('g')
                .long("genesis")
                .value_name("GENESIS")
                .help("Path to the ledger genesis transactions")
        )
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .value_name("NAMESPACE")
                .help("Namespace of ledger for DID resolution. Only needed if not multiple-ledgers")
        )
        .arg(
            Arg::new("multiple-ledgers")
                .action(ArgAction::SetTrue)
                .help("Support multiple ledgers")
                .long("multiple-ledgers")
        )
        .arg(
            Arg::new("host")
                .short('h')
                .long("host")
                .value_name("HOST")
                .default_value("0.0.0.0")
                .help("Set the local address to listen on")
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Sets the local port to listen on")
        )
        .arg(
            Arg::new("no-refresh")
                .long("no-refresh")
                .action(ArgAction::SetTrue)
                .help("Disable initial validator node refresh")
        )
        .arg(
            Arg::new("refresh-interval")
                .short('r')
                .long("refresh-interval")
                .value_name("INTERVAL")
                .help("Set the interval in minutes between validator node refresh attempts (0 to disable refresh, default 120)"),
        )
        .arg(
            Arg::new("tls-cert")
                .long("tls-cert")
                .value_name("CERT")
                .help("Path to the TLS certificate file")
        )
        .arg(
            Arg::new("tls-key")
                .long("tls-key")
                .value_name("KEY")
                .help("Path to the TLS private key file")
        );

    #[cfg(unix)]
    {
        app = app.arg(
            Arg::new("socket")
                .short('s')
                .long("socket")
                .value_name("SOCKET")
                .help("Sets the UNIX socket path listen on"),
        );
    }

    app = app.disable_help_flag(true).arg(
        Arg::new("help")
            .short('?')
            .long("help")
            .help("Display command line parameters")
            .action(ArgAction::Help),
    );

    let matches = app.get_matches();

    let genesis = matches.get_one::<String>("genesis").cloned();

    let namespace = matches
        .get_one::<String>("name")
        .cloned()
        .unwrap_or_else(|| "test".to_string());

    let is_multiple = matches.get_flag("multiple-ledgers");

    if matches.contains_id("socket") {
        if matches.contains_id("host") {
            return Err("Cannot specify both host and socket".to_owned());
        }
    } else if !matches.contains_id("port") {
        return Err("Port number or socket must be specified".to_owned());
    }

    #[cfg(unix)]
    let socket = matches.get_one::<String>("socket").cloned();

    let host = matches.get_one::<String>("host").cloned();
    let port = if let Some(port) = matches.get_one::<String>("port") {
        Some(port.parse::<u16>().map_err(|_| "Invalid port number")?)
    } else {
        None
    };
    let init_refresh = !matches.get_flag("no-refresh");
    let interval_refresh = matches
        .get_one::<String>("refresh-interval")
        .map(|ival| ival.parse::<u32>().map_err(|_| "Invalid refresh interval"))
        .transpose()?
        .unwrap_or(120);

    let tls_cert_path = matches.get_one::<String>("tls-cert").cloned();
    let tls_key_path = matches.get_one::<String>("tls-key").cloned();

    Ok(Config {
        genesis,
        namespace,
        #[cfg(unix)]
        socket,
        host,
        port,
        init_refresh,
        interval_refresh,
        is_multiple,
        tls_cert_path,
        tls_key_path,
    })
}
