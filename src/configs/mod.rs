use std::path::Path;

pub use serde::{Deserialize, Serialize};

macro_rules! def_config_module {
    ($Config:ident {
        $($field:ident:$type:ty $(= $default_value:expr)?),* $(,)?
    }) => {
        #[derive(Serialize, Deserialize, Debug, Clone)]
        pub struct $Config {
            $(pub $field: $type),*
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
        impl Config for $Config {
            fn load_from_env() -> Self {
                $Config {
                    $($field: parse_from_toml(
                        &std::env::var(
                            &format!(
                                "{}_{}",
                                stringify!($Config).to_uppercase(),
                                stringify!($field).to_uppercase()
                            )
                        ).unwrap_or_default()
                    )),*
                }
            }
        }
    };
}

def_config_module! {
    PgConfig {
        host: String = "localhost".to_string(),
        port: u16 = 5432_u16,
        db: String = "selene_bot".to_string(),
        user: String = "postgres".to_string(),
        password: String = "postgres".to_string(),
    }
}

// 暂时用influxdb，以后可能换成cnosDB
def_config_module! {
    InfluxDbConfig {
        url: String = "http://localhost:8086".to_string(),
        database: String = "selene_bot".to_string(),
    }
}

def_config_module! {
    SdkConfig {
        app_id: String,
        token: String,
    }
}

trait Config {
    fn load_from_env() -> Self;
}

fn parse_from_toml<C: Default + for<'a> Deserialize<'a>>(s: &str) -> C {
    toml::from_str(s).unwrap_or_default()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BotConfig {
    pub influxdb: InfluxDbConfig,
    pub pg: PgConfig,
    pub sdk: SdkConfig,
}

impl BotConfig {
    pub fn load_from_file(path: impl AsRef<Path>) -> Self {
        let content = std::fs::read_to_string(path).expect("failed to read config file");
        toml::from_str(&content).expect("failed to parse config file")
    }
    pub fn load_from_env() -> Self {
        <Self as Config>::load_from_env()
    }
}

impl Config for BotConfig {
    fn load_from_env() -> Self {
        BotConfig {
            influxdb: InfluxDbConfig::load_from_env(),
            pg: PgConfig::load_from_env(),
            sdk: SdkConfig::load_from_env(),
        }
    }
}

pub fn config_path() -> String {
    std::env::var("SELENE_CONFIG_PATH").unwrap_or_else(|_| {
        log::warn!("SELENE_CONFIG_PATH not set, use default config path: config.toml");
        "config.toml".to_string()
    })
}
