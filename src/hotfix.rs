use crate::decode::SimpleDecodingResult;
use serde::Serialize;

/// A struct representing the hotfix data, containing URLs and version information.
#[derive(Serialize, Default)]
pub struct Hotfix {
    /// URL for the asset bundle.
    pub asset_bundle_url: String,
    /// URL for the ex resource.
    pub ex_resource_url: String,
    /// URL for the lua resource.
    pub lua_url: String,
    /// URL for the ifix resource.
    pub ifix_url: String,
    /// Version number for the mdk resource.
    pub custom_mdk_res_version: u32,
    /// Version number for the ifix resource.
    pub custom_ifix_version: u32,
}

impl Hotfix {
    /// Create a Hotfix with data from the provided SimpleDecodingResult.
    /// Iterates through the fields and assigns values based on URL patterns.
    /// Returns a `Hotfix` struct populated with the corresponding URL values and versions.
    pub fn create_from_simple_message(proto_dec_result: SimpleDecodingResult) -> Self {
        let mut hotfix = Hotfix::default();

        for field in proto_dec_result.fields {
            let value = field.value.to_string();
            if value.contains("/asb/") {
                hotfix.asset_bundle_url = value;
            } else if value.contains("/design_data/") {
                hotfix.ex_resource_url = value;
            } else if value.contains("/lua/") {
                hotfix.lua_url = value;
            } else if value.contains("/ifix/") {
                hotfix.ifix_url = value;
            }
        }

        hotfix
    }
}
