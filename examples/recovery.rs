use app_json_settings::{ConfigError, ConfigManager};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
struct AppSettings {
    theme: String,
    launch_count: u32,
}

fn main() -> app_json_settings::Result<()> {
    let root = std::env::temp_dir().join("app-json-settings-recovery-example");
    let manager = ConfigManager::<AppSettings>::new()
        .with_root_dir(&root)
        .try_with_filename("settings.json")?;

    // Simulate a settings file left unreadable by an external edit, so this
    // example actually exercises the recovery branch below instead of
    // silently taking the happy path. A real application would not do this;
    // it would simply encounter an already-invalid file.
    std::fs::create_dir_all(&root)?;
    std::fs::write(manager.path(), "not-json")?;

    let settings = match manager.load_or_default() {
        Ok(settings) => settings,
        Err(ConfigError::Deserialize(error)) => {
            eprintln!("settings file is invalid, moving it aside: {error}");

            // The backup carries the same sensitivity as the original
            // settings file. Handle it with the same care you would give
            // the original (see docs/src/save-behavior.md for the Unix
            // permission model this crate applies to the file itself).
            let backup = manager.path().with_extension("json.bak");
            std::fs::rename(manager.path(), &backup)?;
            println!("backed up invalid settings to: {}", backup.display());

            manager.load_or_default()?
        }
        Err(error) => return Err(error),
    };

    println!("settings path: {}", manager.path().display());
    println!("theme: {}", settings.theme);
    println!("launch count: {}", settings.launch_count);

    Ok(())
}
