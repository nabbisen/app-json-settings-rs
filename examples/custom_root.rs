use app_json_settings::ConfigManager;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct PortableSettings {
    recent_files: Vec<String>,
    sidebar_visible: bool,
}

impl Default for PortableSettings {
    fn default() -> Self {
        Self {
            recent_files: Vec::new(),
            sidebar_visible: true,
        }
    }
}

fn main() -> app_json_settings::Result<()> {
    // Caller-provided roots are useful for portable apps, tests, sandboxed
    // hosts, and UWP-style apps where the host resolves its own local folder.
    let root = std::env::temp_dir().join("app-json-settings-custom-root-example");

    let manager = ConfigManager::<PortableSettings>::new()
        .with_root_dir(&root)
        .try_with_filename("preferences.json")?;

    let mut settings = manager.load_or_default()?;

    if settings.recent_files.is_empty() {
        settings
            .recent_files
            .push("/example/documents/readme.md".to_string());
        manager.save(&settings)?;
    }

    println!("custom root: {}", root.display());
    println!("settings file: {}", manager.path().display());
    println!("recent files: {:?}", settings.recent_files);

    Ok(())
}
