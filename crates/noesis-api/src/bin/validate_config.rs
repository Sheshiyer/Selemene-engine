use noesis_api::ApiConfig;
use std::env;

struct EnvVarSpec {
    name: &'static str,
    aliases: &'static [&'static str],
    required_in_production: bool,
    default_behavior: &'static str,
}

const ENV_SPECS: &[EnvVarSpec] = &[
    EnvVarSpec {
        name: "RUST_ENV",
        aliases: &[],
        required_in_production: false,
        default_behavior: "defaults to development mode",
    },
    EnvVarSpec {
        name: "HOST",
        aliases: &["SERVER_HOST"],
        required_in_production: false,
        default_behavior: "defaults to 0.0.0.0",
    },
    EnvVarSpec {
        name: "PORT",
        aliases: &["SERVER_PORT"],
        required_in_production: false,
        default_behavior: "defaults to 8080",
    },
    EnvVarSpec {
        name: "JWT_SECRET",
        aliases: &[],
        required_in_production: true,
        default_behavior: "uses development default outside production",
    },
    EnvVarSpec {
        name: "DATABASE_URL",
        aliases: &[],
        required_in_production: false,
        default_behavior: "auth endpoints disabled when missing",
    },
    EnvVarSpec {
        name: "REDIS_URL",
        aliases: &[],
        required_in_production: false,
        default_behavior: "L2 cache disabled when missing",
    },
    EnvVarSpec {
        name: "ALLOWED_ORIGINS",
        aliases: &[],
        required_in_production: false,
        default_behavior: "defaults to localhost origins",
    },
    EnvVarSpec {
        name: "RATE_LIMIT_REQUESTS",
        aliases: &[],
        required_in_production: false,
        default_behavior: "defaults to 100 requests per window",
    },
    EnvVarSpec {
        name: "RATE_LIMIT_WINDOW_SECS",
        aliases: &[],
        required_in_production: false,
        default_behavior: "defaults to 60 seconds",
    },
    EnvVarSpec {
        name: "REQUEST_TIMEOUT_SECS",
        aliases: &[],
        required_in_production: false,
        default_behavior: "defaults to 30 seconds",
    },
    EnvVarSpec {
        name: "RUST_LOG",
        aliases: &[],
        required_in_production: false,
        default_behavior: "defaults to info,noesis_api=debug",
    },
    EnvVarSpec {
        name: "LOG_FORMAT",
        aliases: &[],
        required_in_production: false,
        default_behavior: "defaults to pretty",
    },
];

fn resolved_env_value(spec: &EnvVarSpec) -> Option<(&'static str, String)> {
    if let Ok(value) = env::var(spec.name) {
        return Some((spec.name, value));
    }

    for alias in spec.aliases {
        if let Ok(value) = env::var(alias) {
            return Some((alias, value));
        }
    }

    None
}

fn main() {
    let dry_run = env::args().skip(1).any(|arg| arg == "--dry-run");
    if !dry_run {
        eprintln!("Usage: cargo run -p noesis-api --bin validate_config -- --dry-run");
        std::process::exit(2);
    }

    println!("Noesis API configuration audit (dry-run)");
    println!("=====================================");

    for spec in ENV_SPECS {
        match resolved_env_value(spec) {
            Some((source, value)) => {
                let rendered = if spec.name == "JWT_SECRET" {
                    format!("<redacted:{} chars>", value.len())
                } else {
                    value
                };
                println!(
                    "OK   {:<22} source={:<12} value={}",
                    spec.name, source, rendered
                );
            }
            None if spec.required_in_production => {
                println!(
                    "WARN {:<22} missing (required only in production); {}",
                    spec.name, spec.default_behavior
                );
            }
            None => {
                println!(
                    "WARN {:<22} missing (optional); {}",
                    spec.name, spec.default_behavior
                );
            }
        }
    }

    let config = match ApiConfig::from_env() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Configuration load failed: {}", err);
            std::process::exit(1);
        }
    };

    if let Err(err) = config.validate() {
        eprintln!("Configuration validation failed: {}", err);
        std::process::exit(1);
    }

    println!(
        "Validation OK: bind_address={} redis={} database={}",
        config.bind_address(),
        if config.redis_url.is_some() {
            "enabled"
        } else {
            "disabled"
        },
        if config.database_url.is_some() {
            "configured"
        } else {
            "degraded"
        }
    );
}
