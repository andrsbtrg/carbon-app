use std::path::PathBuf;

pub struct SettingsProvider {}

impl SettingsProvider {
    const APP_NAME: &'static str = "carbon";
    fn default_path() -> PathBuf {
        #[cfg(unix)]
        let app_data = std::env::var("HOME").expect("No HOME directory");
        #[cfg(windows)]
        let app_data = std::env::var("APP_DATA").expect("No APP_DATA directory");
        let path = std::path::Path::new(&app_data);
        let os = std::env::consts::OS;
        match os {
            "linux" => path.join(".var").join("app").join(Self::APP_NAME),
            "windows" => path.join(&app_data),
            _ => todo!("This os is not supported."),
        }
    }
    pub fn cache_dir() -> PathBuf {
        Self::default_path().join("cache")
    }
}
