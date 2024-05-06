use dotenvy::dotenv;
use regex::Regex;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub db_url: String,
    pub listen_on: String,
}

fn validate(c: &Config) -> Result<(), &str> {
    if c.db_url.is_empty() {
        return Err("db_url must be set");
    }

    // Only in-memory SQLite will be implemented for a while.
    if c.db_url != ":memory:" {
        return Err("Only \":memory:\" is supported for DB_URL at this time.");
    }

    if c.listen_on.is_empty() {
        return Err("listen_on must be set (IP:port).");
    }

    let re = Regex::new(":[0-9]+$").unwrap();

    if !re.is_match(&c.listen_on) {
        return Err("listen_on must be an \"IP:port\" string. Use \
            \"127.0.0.1:port\" or \"[::]:port\" for localhost. Use \
            \"0.0.0.0:port\" for all interfaces.");
    }

    Ok(())
}

pub fn load() -> Config {
    // Read the .env file (if it exists) into the environment.
    dotenv().ok();

    // Get the database URL and listen address(es) from the environment.
    let conf = match envy::prefixed("CURTAILING_").from_env::<Config>() {
        Ok(c) => c,
        Err(e) => panic!("Config error: {}", e),
    };

    match validate(&conf) {
        Ok(_) => conf,
        Err(e) => panic!("Config error: {}", e),
    }
}
