use std::io::{Cursor, Read};
use varint_rs::VarintReader;

/// Opens a file dialog to allow the user to select a folder.
/// Returns the selected folder's path, or `None` if the selection is canceled.
pub fn select_folder() -> Option<std::path::PathBuf> {
    rfd::FileDialog::new()
        .set_directory(".")
        .set_title("Select HSR Folder")
        .pick_folder()
}

/// Returns the path to the "BinaryVersion.bytes" file located under "StarRail_Data/StreamingAssets" from the given base path.
pub fn get_binary_version_path(base: &std::path::PathBuf) -> std::path::PathBuf {
    base.join("StarRail_Data/StreamingAssets/BinaryVersion.bytes")
}

/// Returns the path to the "ClientConfig.bytes" file located under "StarRail_Data/StreamingAssets" from the given base path.
pub fn get_client_config_path(base: &std::path::PathBuf) -> std::path::PathBuf {
    base.join("StarRail_Data/StreamingAssets/ClientConfig.bytes")
}

pub trait CursorExt {
    type Error;
    fn read_string(&mut self) -> Result<String, Self::Error>;
    fn read_bool(&mut self) -> Result<bool, Self::Error>;
    fn read_u32_be(&mut self) -> Result<u32, Self::Error>;
    fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, Self::Error>;
}

impl CursorExt for Cursor<Vec<u8>> {
    type Error = std::io::Error;

    fn read_bool(&mut self) -> Result<bool, Self::Error> {
        let mut byte = [0; 1];
        self.read_exact(&mut byte)?;
        Ok(byte[0] != 0)
    }

    fn read_string(&mut self) -> Result<String, Self::Error> {
        self.read_bool()?;
        let length = self.read_u32_varint()? as usize;
        let mut buffer = vec![0u8; length];
        self.read_exact(&mut buffer)?;
        return Ok(String::from_utf8_lossy(&buffer).to_string());
    }

    fn read_u32_be(&mut self) -> Result<u32, Self::Error> {
        let mut buffer = [0u8; 4];
        self.read_exact(&mut buffer)?;
        Ok(u32::from_be_bytes(buffer))
    }

    fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, Self::Error> {
        let mut buffer = vec![0u8; len];
        self.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}
