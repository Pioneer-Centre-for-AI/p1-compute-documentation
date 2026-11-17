use std::fs;

use crate::check::clusters::{CLUSTER_IDS, ClusterId};
use crate::context::Ctx;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClusterSettings {
    pub user: Option<String>,
    pub alias: Option<String>,
    pub sync: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DesktopSettings {
    pub open: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub dtu: Option<ClusterSettings>,
    pub gefion: Option<ClusterSettings>,
    pub ngc: Option<ClusterSettings>,
    pub desktop: Option<DesktopSettings>,
}

impl Config {
    pub fn cluster(&self, id: ClusterId) -> Option<&ClusterSettings> {
        match id {
            ClusterId::Dtu => self.dtu.as_ref(),
            ClusterId::Gefion => self.gefion.as_ref(),
            ClusterId::Ngc => self.ngc.as_ref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError(pub String);

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ConfigError {}

fn toml_error(e: &toml::de::Error, text: &str) -> String {
    let message = e.message().trim().replace('\n', "; ");
    match e.span() {
        Some(span) => {
            let before = &text[..span.start.min(text.len())];
            let line = before.matches('\n').count() + 1;
            let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
            format!("TOML parse error at line {line}, column {column}: {message}")
        }
        None => format!("TOML parse error: {message}"),
    }
}

pub fn parse_config(text: &str, file: &str) -> Result<Config, ConfigError> {
    let raw: toml::Table = toml::from_str(text)
        .map_err(|e| ConfigError(format!("{file}: {}", toml_error(&e, text))))?;
    let mut config = Config::default();
    for (key, value) in raw {
        if key == "desktop" {
            config.desktop = Some(parse_desktop(&value, file)?);
            continue;
        }
        let Some(id) = ClusterId::parse(&key) else {
            let ids = CLUSTER_IDS.iter().map(|c| c.as_str()).collect::<Vec<_>>().join(", ");
            return Err(ConfigError(format!(
                "{file}: unknown table [{key}]; expected {ids} or desktop"
            )));
        };
        let Some(table) = value.as_table() else {
            return Err(ConfigError(format!("{file}: {key} must be a table, as in [{key}]")));
        };
        let mut settings = ClusterSettings::default();
        for (field, v) in table {
            if field == "sync" {
                let Some(b) = v.as_bool() else {
                    return Err(ConfigError(format!("{file}: {key}.sync must be true or false")));
                };
                settings.sync = Some(b);
                continue;
            }
            if field != "user" && field != "alias" && field != "account" {
                return Err(ConfigError(format!(
                    "{file}: unknown setting {key}.{field}; expected user, alias or sync"
                )));
            }
            let s = match v.as_str() {
                Some(s) if !s.is_empty() => s.to_string(),
                _ => {
                    return Err(ConfigError(format!(
                        "{file}: {key}.{field} must be a non-empty string"
                    )));
                }
            };
            match field.as_str() {
                "user" => settings.user = Some(s),
                "alias" => settings.alias = Some(s),
                _ => {}
            }
        }
        match id {
            ClusterId::Dtu => config.dtu = Some(settings),
            ClusterId::Gefion => config.gefion = Some(settings),
            ClusterId::Ngc => config.ngc = Some(settings),
        }
    }
    Ok(config)
}

fn parse_desktop(value: &toml::Value, file: &str) -> Result<DesktopSettings, ConfigError> {
    let Some(table) = value.as_table() else {
        return Err(ConfigError(format!("{file}: desktop must be a table, as in [desktop]")));
    };
    let mut settings = DesktopSettings::default();
    for (field, v) in table {
        if field != "open" {
            return Err(ConfigError(format!(
                "{file}: unknown setting desktop.{field}; expected open"
            )));
        }
        match v.as_str() {
            Some(s) if s.contains("{host}") => settings.open = Some(s.to_string()),
            _ => {
                return Err(ConfigError(format!(
                    "{file}: desktop.open must be a command with {{host}} in it"
                )));
            }
        }
    }
    Ok(settings)
}

pub fn load_config(file: &str) -> Result<Config, ConfigError> {
    match fs::read_to_string(file) {
        Ok(text) => parse_config(&text, file),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(ConfigError(format!("{file}: {e}"))),
    }
}

pub fn load_config_or_say(ctx: &Ctx) -> Option<Config> {
    match load_config(&ctx.paths.config_file) {
        Ok(c) => Some(c),
        Err(e) => {
            ctx.err.write(&format!("aicentre: {e}\n"));
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_settings_and_names_mistakes() {
        let c = parse_config(
            "[dtu]\nuser = \"s1\"\nsync = false\n[desktop]\nopen = \"zed ssh://{host}{urlpath}\"\n",
            "/c.toml",
        )
        .unwrap();
        assert_eq!(c.dtu.as_ref().unwrap().user.as_deref(), Some("s1"));
        assert_eq!(c.dtu.as_ref().unwrap().sync, Some(false));
        assert_eq!(c.desktop.unwrap().open.as_deref(), Some("zed ssh://{host}{urlpath}"));
        let err = |t: &str| parse_config(t, "/c.toml").unwrap_err().0;
        assert_eq!(
            err("[foo]\n"),
            "/c.toml: unknown table [foo]; expected dtu, gefion, ngc or desktop"
        );
        assert_eq!(err("dtu = 1\n"), "/c.toml: dtu must be a table, as in [dtu]");
        assert_eq!(err("[dtu]\nsync = \"no\"\n"), "/c.toml: dtu.sync must be true or false");
        assert_eq!(
            err("[gefion]\nuser = \"\"\n"),
            "/c.toml: gefion.user must be a non-empty string"
        );
        assert_eq!(
            err("[dtu]\nhost = \"x\"\n"),
            "/c.toml: unknown setting dtu.host; expected user, alias or sync"
        );
        assert_eq!(
            err("[desktop]\nopen = \"zed\"\n"),
            "/c.toml: desktop.open must be a command with {host} in it"
        );
        assert_eq!(
            err("[desktop]\nrun = \"x\"\n"),
            "/c.toml: unknown setting desktop.run; expected open"
        );
        assert_eq!(err("desktop = 3\n"), "/c.toml: desktop must be a table, as in [desktop]");
        assert!(err("[dtu\n").starts_with("/c.toml: TOML parse error at line 1"));
    }
}
