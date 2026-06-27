use app_json_settings::ConfigManager;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct AppSettings {
    theme: String,
    window_width: u32,
    window_height: u32,
    show_welcome_tip: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            window_width: 1024,
            window_height: 768,
            show_welcome_tip: true,
        }
    }
}

fn main() -> app_json_settings::Result<()> {
    // The production desktop form is usually:
    //
    // let manager = ConfigManager::<AppSettings>::for_app("my-gui-app")?;
    //
    // This example uses a temporary root so running it does not write to your
    // real app settings directory.
    let root = std::env::temp_dir().join("app-json-settings-basic-example");
    let manager = ConfigManager::<AppSettings>::new()
        .with_root_dir(root)
        .try_with_filename("settings.json")?;

    let settings = manager.load_or_default()?;

    println!("settings path: {}", manager.path().display());
    println!("theme: {}", settings.theme);
    println!(
        "window: {}x{}",
        settings.window_width, settings.window_height
    );

    Ok(())
}
