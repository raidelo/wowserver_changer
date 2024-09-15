use clap::{Arg, Command};
use std::process::exit;
use wowserver::{get_config, get_server_from, save_server, verify_config};

fn main() {
    let cli = Command::new("Wowserver").arg(
        Arg::new("servidor")
            .required(true)
            .help("el nombre del servidor")
            .action(clap::ArgAction::Set),
    );

    let args = cli.get_matches();

    let server: &String = args.get_one("servidor").unwrap_or_else(|| {
        eprintln!("error: el parámetro posicional <servidor> es obligatorio");
        exit(1)
    });

    let config: toml::map::Map<String, toml::Value> = match get_config() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("error: {err}");
            exit(1)
        }
    };

    if let Err(error_data) = verify_config(&config) {
        eprintln!("error: {error_data}");
        exit(1)
    };

    let content = format!(
        "set realmlist {}",
        get_server_from(server, &config).unwrap_or_else(|| {
            eprintln!("error: valor `{server}` no existente en el archivo de configuración");
            exit(1)
        })
    );

    match save_server(&content, &config) {
        Ok(val) => {
            val.print_data();
        }
        Err(err) => eprintln!("error: {err}"),
    }
}
