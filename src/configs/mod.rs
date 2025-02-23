use std::path::Path;

pub use serde::{Deserialize, Serialize};

macro_rules! def_module_config {
    ($Config:ident {
        $(
            $(#[$attr:meta])*
            $field:ident:$type:ty $(= $default_value:expr)?
        ),* $(,)?
    }) => {
        #[derive(Serialize, Deserialize, Debug, Clone)]
        #[serde(default)]
        pub struct $Config {
            $(
                $(#[$attr])*
                pub $field: $type
            ),*
        }
        impl Default for $Config {
            fn default() -> Self {
                #[allow(unreachable_code)]
                $Config {
                    $($field: 'val_block: {
                        $(break 'val_block $default_value.into();)?
                        break 'val_block Default::default();
                    }),*
                }
            }
        }
        impl ModuleConfig for $Config {
            fn load_from_env() -> Self {
                $Config {
                    $($field:
                        std::env::var(&format!(
                            "{}_{}",
                            stringify!($Config).to_uppercase(),
                            stringify!($field).to_uppercase()
                        )).as_deref().map(parse_from_toml::<$type>).unwrap_or_default()
                    ),*
                }
            }
        }
    };
}

def_module_config! {
    PgConfig {
        host: String = "localhost",
        port: u16 = 5432_u16,
        db: String = "selene_bot",
        user: String = "postgres",
        password: String = "postgres",
    }
}

// 暂时用influxdb，以后可能换成cnosDB
def_module_config! {
    InfluxDbConfig {
        url: String = "http://localhost:8086",
        database: String = "selene_bot",
    }
}

def_module_config! {
    SdkConfig {
        app_id: String,
        secret: String,
    }
}

def_module_config! {
    SurrealConfig {
        host: String = "localhost",
        port: u16 = 8080_u16,
        namespace: String = "selene_bot",
        username: String = "selene_bot",
        password: String = "selene_bot",
    }
}
trait ModuleConfig {
    fn load_from_env() -> Self;
}

fn parse_from_toml<C: Default + for<'a> Deserialize<'a>>(s: &str) -> C {
    toml::from_str(s).unwrap_or_default()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BotConfig {
    pub influxdb: InfluxDbConfig,
    pub pg: PgConfig,
    pub sdk: SdkConfig,
    pub surreal: SurrealConfig,
}

impl BotConfig {
    pub fn load_from_file(path: impl AsRef<Path>) -> Self {
        let content = std::fs::read_to_string(path).expect("failed to read config file");
        toml::from_str(&content).expect("failed to parse config file")
    }
    pub fn load_from_env() -> Self {
        BotConfig {
            influxdb: InfluxDbConfig::load_from_env(),
            pg: PgConfig::load_from_env(),
            sdk: SdkConfig::load_from_env(),
            surreal: SurrealConfig::load_from_env(),
        }
    }
}

pub fn config_path() -> String {
    std::env::var("SELENE_CONFIG_PATH").unwrap_or_else(|_| {
        log::warn!("SELENE_CONFIG_PATH not set, use default config path: config.toml");
        "config.toml".to_string()
    })
}
