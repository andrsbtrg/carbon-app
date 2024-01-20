use std::path::PathBuf;

pub struct SettingsProvider {}

impl SettingsProvider {
    const APP_NAME: &'static str = "carbon";
    pub fn default_path() -> PathBuf {
        #[cfg(unix)]
        let app_data = std::env::var("HOME").expect("No HOME directory");
        #[cfg(windows)]
        let app_data = std::env::var_os("APPDATA").expect("No APP_DATA directory");
        let path = std::path::Path::new(&app_data);
        let os = std::env::consts::OS;
        match os {
            "linux" => path.join(".var").join("app").join(Self::APP_NAME),
            "windows" => path.join(&app_data).join(Self::APP_NAME),
            _ => todo!("This os is not supported."),
        }
    }
    pub fn api_key_path() -> PathBuf {
        Self::default_path().join("api_key.txt")
    }
    pub fn cache_dir() -> PathBuf {
        Self::default_path().join("cache")
    }
}

pub fn set_api_key(key: &str) {
    let path = SettingsProvider::api_key_path();
    std::fs::write(path, key).expect("ERROR: not possible to write to default path");
}
