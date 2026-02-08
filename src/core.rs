use serde::{de::DeserializeOwned, Serialize};

use std::fs;
use std::io;
use std::marker::PhantomData;
use std::path::PathBuf;

use crate::core::constant::DEFAULT_FILE_NAME;
use crate::core::dir::default_config_dir;
use crate::core::json::JsonFormat;
use crate::ConfigError;
use crate::Result;

pub mod constant;
mod dir;
pub mod error;
mod json;

pub struct ConfigManager<T> {
    //     app_name: String,
    folder_path: PathBuf,
    file_name: String,
    json_format: JsonFormat,
    _marker: PhantomData<T>,
}

impl<T> ConfigManager<T>
where
    T: Serialize + DeserializeOwned,
{
    /// デフォルト初期化
    /// フォルダ: OS 標準 config ディレクトリ / app_name
    pub fn new(app_name: &str) -> Self {
        let mut path = default_config_dir();
        path.push(app_name);

        Self {
            // app_name: app_name.to_string(),
            folder_path: path,
            file_name: DEFAULT_FILE_NAME.to_string(),
            json_format: JsonFormat::Pretty,
            _marker: PhantomData,
        }
    }

    /// カレントディレクトリに保存
    pub fn at_current_dir(mut self) -> Self {
        self.folder_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        self
    }

    /// 任意パスに保存
    pub fn at_custom_dir<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.folder_path = path.into();
        self
    }

    /// ファイル名変更
    pub fn with_filename(mut self, name: &str) -> Self {
        self.file_name = name.to_string();
        self
    }

    /// JSON を pretty 形式で保存
    pub fn disable_pretty_json(mut self) -> Self {
        self.json_format = JsonFormat::Compact;
        self
    }

    /// フルパス取得
    pub fn path(&self) -> PathBuf {
        self.folder_path.join(&self.file_name)
    }

    /// 完全保存（置換書き込み）
    pub fn save(&self, config: &T) -> Result<()> {
        if !self.folder_path.exists() {
            fs::create_dir_all(&self.folder_path)?;
        }

        let content = match self.json_format {
            JsonFormat::Compact => serde_json::to_string(config)?,
            JsonFormat::Pretty => serde_json::to_string_pretty(config)?,
        };

        fs::write(self.path(), content)?;
        Ok(())
    }

    /// ファイルが存在する前提のロード
    pub fn load(&self) -> Result<T> {
        let content = fs::read_to_string(self.path())?;
        Ok(serde_json::from_str(&content)?)
    }
}

//
// Default 対応 API
//

impl<T> ConfigManager<T>
where
    T: Serialize + DeserializeOwned + Default,
{
    /// 存在しなければ default を生成して保存
    pub fn load_or_default(&self) -> Result<T> {
        let path = self.path();

        match fs::read_to_string(&path) {
            Ok(content) => Ok(serde_json::from_str(&content)?),

            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let default_config = T::default();
                self.save(&default_config)?;
                Ok(default_config)
            }

            Err(e) => Err(ConfigError::Io(e)),
        }
    }

    /// 安全な read-modify-write
    /// save() と並ぶ主要 API
    pub fn update<F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut T),
    {
        let mut cfg = self.load_or_default()?;
        f(&mut cfg);
        self.save(&cfg)?;
        Ok(cfg)
    }
}
