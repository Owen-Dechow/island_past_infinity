use std::{fmt::Display, path::Path};

use macroquad::texture::{load_texture, Texture2D};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum AssetManageError {
    Macro(macroquad::Error),
    Serde(serde_json::Error),
    Io(std::io::Error),
}

impl Display for AssetManageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetManageError::Macro(error) => write!(f, "{error}"),
            AssetManageError::Serde(error) => write!(f, "{error}"),
            AssetManageError::Io(error) => write!(f, "{error}"),
        }
    }
}

impl From<macroquad::Error> for AssetManageError {
    fn from(value: macroquad::Error) -> Self {
        Self::Macro(value)
    }
}

impl From<serde_json::Error> for AssetManageError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

impl From<std::io::Error> for AssetManageError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub type AssetManageResult<T> = Result<T, AssetManageError>;

pub fn serialize<T, P>(obj: &T, path: P) -> AssetManageResult<()>
where
    T: Serialize,
    P: AsRef<Path>,
{
    std::fs::write(path, serde_json::to_string_pretty(obj)?)?;
    return Ok(());
}

pub fn deserialize<T, P>(path: P) -> AssetManageResult<T>
where
    T: for<'de> Deserialize<'de>,
    P: AsRef<Path>,
{
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}

pub async fn load_tex_with_meta<T, P>(path: P) -> AssetManageResult<(T, Texture2D)>
where
    T: for<'de> Deserialize<'de>,
    P: AsRef<Path>,
{
    let path = &path.as_ref().to_string_lossy();
    let tex = load_texture(path).await?;
    tex.set_filter(macroquad::texture::FilterMode::Nearest);

    let meta = deserialize(format!("{path}.meta.json"))?;

    return Ok((meta, tex));
}
