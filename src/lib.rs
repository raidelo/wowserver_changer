use std::{collections::HashMap, env, fs, io, path};

const CONFIG_FILE: &str = "config.toml";
const REALMLIST_FILE: &str = "realmlist.wtf";

/// Función encargada de leer el archivo de configuración
fn read_config() -> Result<String, io::Error> {
    let content = fs::read_to_string(env::args().next().unwrap() + "\\..\\" + CONFIG_FILE)?;
    Ok(content)
}

/// Función encargada de leer el archivo de configuración y parsearlo como un struct de tipo toml::Table
pub fn get_config() -> Result<toml::Table, String> {
    let content = match read_config() {
        Ok(data) => data,
        Err(err) => {
            match err.kind() {
                io::ErrorKind::NotFound => {
                    return Err("el fichero de configuración no existe".to_string())
                }
                io::ErrorKind::PermissionDenied => {
                    return Err(
                        "no tiene permisos para abrir el fichero de configuración".to_string()
                    )
                }
                err => return Err(err.to_string()),
            };
        }
    };

    let table = match toml::from_str(&content) {
        Ok(data) => data,
        Err(err) => return Err(err.message().to_string()),
    };

    Ok(table)
}

/// Función encargada de verificar que archivo de configuración cumpla con el formato necesario
pub fn verify_config(config: &toml::Table) -> Result<(), String> {
    let mut missing_section: bool = false;
    let mut invalid_format: bool = true;

    match config.get("PATHS") {
        Some(val) => match val {
            toml::Value::Array(arr) => match arr.get(0) {
                Some(val) => match val {
                    toml::Value::String(_) => invalid_format = false,
                    _ => (),
                },
                None => (),
            },
            _ => (),
        },
        None => missing_section = true,
    }

    if missing_section {
        return Err(format!(
            "sección <PATHS> faltante en el archivo de configuración"
        ));
    } else if invalid_format {
        return Err(format!(
            "formato inválido en la variable <PATHS> en el archivo de configuración"
        ));
    }

    match config.get("SERVERS") {
        Some(val) => match val {
            toml::Value::Table(tab) => match tab.values().next() {
                Some(val) => match val {
                    toml::Value::String(_) => invalid_format = false,
                    _ => (),
                },
                None => (),
            },
            _ => (),
        },
        None => missing_section = true,
    }

    if missing_section {
        return Err(format!(
            "sección <SERVERS> faltante en el archivo de configuración"
        ));
    } else if invalid_format {
        return Err(format!(
            "formato inválido en la sección <SERVERS> en el archivo de configuración"
        ));
    }

    Ok(())
}

/// Función encargada de obtener el valor correspondiente a una clave en la tabla sin importar mayúsculas y minúsculas
fn get_value<'a>(servers_section: &'a toml::Table, key: &str) -> Option<&'a toml::Value> {
    for k in servers_section.keys() {
        if k.to_lowercase() == key.to_lowercase() {
            return Some(servers_section.get(k).unwrap());
        }
    }
    None
}

/// Función encargada de intentar obtener el servidor del archivo de configuración a partir de su clave, o el mismo servidor en caso contrario si es que aparece en los valores
pub fn get_server_from(server: &str, config: &toml::Table) -> Option<String> {
    if let Some(val) = config.get("SERVERS") {
        if let toml::Value::Table(tab) = val {
            match get_value(tab, server) {
                Some(val) => {
                    return Some(
                        val.as_str()
                            .expect("los valores deberían ser de tipo String")
                            .to_string(),
                    )
                }
                None => {
                    match tab
                        .values()
                        .map(|x| x.as_str().expect("los valores deberían ser de tipo String"))
                        .collect::<Vec<&str>>()
                        .contains(&server)
                    {
                        true => return Some(server.to_string()),
                        false => return None,
                    }
                }
            }
        }
    }
    return None;
}

pub struct ModifiedFilesResult {
    ok: Vec<path::PathBuf>,
    err: HashMap<path::PathBuf, String>,
}

impl ModifiedFilesResult {
    fn new() -> Self {
        Self {
            ok: Vec::<path::PathBuf>::new(),
            err: HashMap::<path::PathBuf, String>::new(),
        }
    }

    pub fn print_data(&self) {
        if self.ok.len() > 0 {
            println!("ok: [");
            for ok_val in self.ok.iter() {
                println!("    {}", ok_val.display());
            }
            println!("]")
        }
        if self.err.len() > 0 {
            println!("error: {{");
            for err_val in self.err.iter() {
                println!("    {}", err_val.1);
            }
            println!("}}")
        }
    }
}

/// Función encargada de guardar el contenido dado en el archivo realmlist.wtf
pub fn save_server(content: &str, config: &toml::Table) -> Result<ModifiedFilesResult, String> {
    let std_err =
        "debería haber una sección <PATHS> de tipo Vec<String> en el archivo de configuración"
            .to_string();
    let paths = match config.get("PATHS") {
        Some(val) => match val {
            toml::Value::Array(arr) => arr,
            _ => return Err(std_err),
        },
        None => return Err(std_err),
    };

    let mut data = ModifiedFilesResult::new();

    for path in paths {
        if let Some(target) = path.as_str() {
            let dir = path::Path::new(target);

            match dir.is_dir() {
                true => {
                    let file = dir.join(REALMLIST_FILE);
                    if let Err(err) = fs::write(&file, content) {
                        match err.kind() {
                            io::ErrorKind::NotFound => {
                                data.err.insert(file, format!("ruta inválida: `{target}`"));
                            }
                            io::ErrorKind::PermissionDenied => {
                                data.err.insert(file.clone(), format!("no tiene permisos suficientes para modificar el archivo `{}`", file.display()));
                            }
                            err => {
                                data.err.insert(file, err.to_string());
                                ()
                            }
                        }
                    } else {
                        data.ok.push(file);
                    }
                }
                false => {
                    data.err
                        .insert(dir.into(), format!("ruta inválida: `{target}`"));
                    ()
                }
            }
        } else {
            return Err(std_err);
        }
    }
    Ok(data)
}
