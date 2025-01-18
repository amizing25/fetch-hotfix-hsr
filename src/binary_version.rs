use crate::util::CursorExt as _;
use std::io::Cursor;

#[derive(Debug)]
#[allow(unused)]
pub struct BinaryVersionData {
    pub branch: String,
    pub revision: u32,
    pub major_version: u32,
    pub minor_version: u32,
    pub patch_version: u32,
    _unk: Vec<u8>, // 15 * sizeof(u32)
    // pub unk2: u32,
    // pub unk1: u32,
    // pub unk1: u32,
    // pub unk1: u32,
    // pub unk1: u32,
    // pub unk1: u32,
    // pub unk1: u32,
    // pub unk1: u32,
    // pub unk1: u32,
    // pub unk1: u32,
    // pub unk1: u32,
    // pub unk1: u32,
    // pub unk1: u32,
    // pub unk1: u32,
    pub time: String,
    pub pak_type: String,
    pub pak_type_detail: String,
    pub start_asset: String,
    pub start_design_data: String,
    pub dispatch_seed: String,
    pub version_string: String,
    pub version_hash: String,
    pub game_core_version: u32,
    pub is_enable_exclude_asset: bool,
    pub sdk_ps_client_id: String,
}

impl BinaryVersionData {
    pub fn get_server_pak_type_version(&self) -> Option<String> {
        for segment in self.version_string.split('-') {
            if segment
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.')
                && segment.contains('.')
                && segment.chars().filter(|&c| c == '.').count() == 2
            {
                return Some(segment.to_string());
            }
        }
        None
    }
}

impl TryFrom<Vec<u8>> for BinaryVersionData {
    type Error = std::io::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, std::io::Error> {
        let mut reader = Cursor::new(value);

        Ok(Self {
            branch: reader.read_string()?,
            revision: reader.read_u32_be()?,
            major_version: reader.read_u32_be()?,
            minor_version: reader.read_u32_be()?,
            patch_version: reader.read_u32_be()?,
            _unk: reader.read_bytes(4 * 15)?,
            time: reader.read_string()?,
            pak_type: reader.read_string()?,
            pak_type_detail: reader.read_string()?,
            start_asset: reader.read_string()?,
            start_design_data: reader.read_string()?,
            dispatch_seed: reader.read_string()?,
            version_string: reader.read_string()?,
            version_hash: reader.read_string()?,
            game_core_version: reader.read_u32_be()?,
            is_enable_exclude_asset: reader.read_bool()?,
            sdk_ps_client_id: reader.read_string()?,
        })
    }
}
