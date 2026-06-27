use app_json_settings::ConfigManager;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct AppSettings {
    launch_count: u32,
    theme: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            launch_count: 0,
            theme: "system".to_string(),
        }
    }
}

fn main() -> app_json_settings::Result<()> {
    let root = std::env::temp_dir().join("app-json-settings-update-example");
    let manager = ConfigManager::<AppSettings>::new()
        .with_root_dir(root)
        .try_with_filename("settings.json")?;

    let updated = manager.update(|settings| {
        settings.launch_count += 1;
        if settings.theme.is_empty() {
            settings.theme = "system".to_string();
        }
    })?;

    println!("settings path: {}", manager.path().display());
    println!("launch count: {}", updated.launch_count);
    println!("theme: {}", updated.theme);

    Ok(())
}
