use varint_rs::VarintReader;

use crate::util::CursorExt as _;
use std::io::{Cursor, Read};

#[derive(Debug)]
#[allow(unused)]
pub struct ClientStartupConfig {
    pub channel_name: String,
    pub bundle_identifier: String,
    pub product_name: String,
    pub script_defines: String,
    pub global_dispatch_url_list: Vec<String>,
}

impl TryFrom<Vec<u8>> for ClientStartupConfig {
    type Error = std::io::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, std::io::Error> {
        let mut reader = Cursor::new(value);

        Ok(Self {
            channel_name: reader.read_string()?,
            bundle_identifier: reader.read_string()?,
            product_name: reader.read_string()?,
            script_defines: reader.read_string()?,
            global_dispatch_url_list: {
                let mut buf = [0; 3]; // TODO!
                reader.read_exact(&mut buf)?;
                (0..reader.read_u32_varint()?)
                    .filter_map(|_| reader.read_string().ok())
                    .collect()
            },
        })
    }
}
