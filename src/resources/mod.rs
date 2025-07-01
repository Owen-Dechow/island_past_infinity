use std::fmt::Display;

pub mod levels;
pub mod sprites;
pub mod tilesets;

#[derive(Debug)]
pub enum AssetLoadError {
    Macro(macroquad::Error),
    Serde(serde_json::Error),
    Io(std::io::Error)
}

impl Display for AssetLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetLoadError::Macro(error) => write!(f, "{error}"),
            AssetLoadError::Serde(error) => write!(f, "{error}"),
            AssetLoadError::Io(error) => write!(f, "{error}"),
        }
    }
}

impl From<macroquad::Error> for AssetLoadError {
    fn from(value: macroquad::Error) -> Self {
        Self::Macro(value)
    }
}

impl From<serde_json::Error> for AssetLoadError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

impl From<std::io::Error> for AssetLoadError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
