use dbus::arg::{prop_cast, PropMap};
use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
use dbus::blocking::Connection;
use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Duration;
use regex::Regex;

const CONFIG_FILE_NAME: &str = ".spotify-status";

const SPOTIFY_DBUS_DEST: &str = "org.mpris.MediaPlayer2.spotify";
const MEDIA_INTERFACE_PATH: &str = "/org/mpris/MediaPlayer2";
const MPRIS_MEDIA_INTERFACE: &str = "org.mpris.MediaPlayer2.Player";
const MEDIA_METADATA_PROP: &str = "Metadata";
const TITLE_PROPERTY: &str = "xesam:title";
const ARTISTS_PROPERTY: &str = "xesam:artist";

const ERR_NO_HOME_DIR: &str = "Error; could not find the home directory of the current user";
const ERR_UNABLE_TO_OPEN_CONFIG_FILE_BUT_EXISTS: &str =
    "Error; unable to open the config file but it exists";

const SPOTIFY_ICON_AWESOME_FONTS: &str = "&#xf1bc;";
const DEFAULT_COLOR: &str = "white";
const DEFAULT_MAX_LENGTH: usize = 45;
const DEFAULT_REMOVE_FEAT : bool = false;
const DEFAULT_FEAT_REGEX : &str = r"\(feat\. [\w* ]*\)";

#[derive(Deserialize)]
struct Config {
    icon: Option<String>,
    color: Option<String>,
    max_length: Option<usize>,
    remove_feat : Option<bool>,
    feat_regex : Option<String>
}

impl Config {
    /// Default configuration
    fn default() -> Config {
        Config {
            icon: Some(SPOTIFY_ICON_AWESOME_FONTS.to_string()),
            color: Some(DEFAULT_COLOR.to_string()),
            max_length: Some(DEFAULT_MAX_LENGTH),
            remove_feat: Some(DEFAULT_REMOVE_FEAT),
            feat_regex: Some(DEFAULT_FEAT_REGEX.to_string()),
        }
    }
}

fn resolve_config() -> std::result::Result<Config, Box<dyn std::error::Error>> {
    let home_dir = home::home_dir();

    if home_dir.is_none() {
        return Err(ERR_NO_HOME_DIR)?;
    }

    let config_path = home_dir.unwrap().join(CONFIG_FILE_NAME);

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let file = File::open(config_path).expect(ERR_UNABLE_TO_OPEN_CONFIG_FILE_BUT_EXISTS);

    let mut reader = BufReader::new(file);
    let mut contents = String::new();

    reader.read_to_string(&mut contents)?;

    let config: Config = toml::from_str(&contents)
        .map_err(|err| -> String { format!("Failed to parse the configuration file: {err}") })?;

    Ok(config)
}

#[test]
fn test_trim_to_length_short() -> Result<(), String> {
    let less_than_30 = "hello";
    assert_eq!(trim_to_length(less_than_30, 30).len(), 5);
    Ok(())
}

#[test]
fn test_trim_to_length_limit() -> Result<(), String> {
    let exactly_30 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    assert_eq!(trim_to_length(exactly_30, 30).len(), 30);
    Ok(())
}

#[test]
fn test_trim_to_length_above_limit() -> Result<(), String> {
    let more_than_30 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    assert_eq!(trim_to_length(more_than_30, 30).len(), 30);
    Ok(())
}

#[test]
fn test_feat_1() -> Result<(), String> {
    let title = "1x1 (feat. Nova Twins)";
    let mut config = Config::default();
    config.remove_feat = Some(true);
    assert_eq!(remove_feat(title, &config), "1x1");
    Ok(())
}

fn remove_feat(title : &str, config : &Config) -> String {
    if !config.remove_feat.unwrap_or(false) {
        return title.to_string();
    }

    let regex = config.feat_regex.as_deref().unwrap_or(DEFAULT_FEAT_REGEX);
    let re = Regex::new(regex).unwrap();
    let cleaned = re.replace_all(title, "");

    return cleaned.trim().to_string();
}

fn trim_to_length(input: &str, max_length: usize) -> String {
    let original_str_len = input.len();

    if original_str_len <= max_length {
        return String::from(input);
    }

    let diff = original_str_len - max_length + 3;
    let mid_ish = original_str_len / 2;

    let pre = &input[..mid_ish - diff / 2];
    let post = &input[mid_ish + diff / 2..];

    format!("{pre}...{post}")
}

fn format_for_printing(config: &Config, display_str: &str) -> String {
    let icon = config.icon.as_deref().unwrap_or(SPOTIFY_ICON_AWESOME_FONTS);
    let color = config.color.as_deref().unwrap_or(DEFAULT_COLOR);
    let max_length = config.max_length.unwrap_or(DEFAULT_MAX_LENGTH);

    // TODO: html escape / encode
    let sized_display_str = trim_to_length(display_str, max_length);
    let displayable_text = html_escape::encode_text(&sized_display_str); 

    return format!("<span color=\"{color}\">{icon} {displayable_text}</span>");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::new_session()?;

    let spotify_dbus_proxy = conn.with_proxy(
        SPOTIFY_DBUS_DEST,
        MEDIA_INTERFACE_PATH,
        Duration::from_millis(5000),
    );

    let config = resolve_config()?;

    let metadata: PropMap = spotify_dbus_proxy.get(MPRIS_MEDIA_INTERFACE, MEDIA_METADATA_PROP)?;

    let title_from_spotify: &String = prop_cast(&metadata, TITLE_PROPERTY).unwrap();
    let title = remove_feat(title_from_spotify, &config);

    let artists: &Vec<String> = prop_cast(&metadata, ARTISTS_PROPERTY).unwrap();

    let contents = format!("{title} (by {})", artists[0]);

    print!("{}", format_for_printing(&config, &contents));

    Ok(())
}
