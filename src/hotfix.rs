use crate::{
    decode::{DecodedValue, DecodingResult, WireType},
    proto::Dispatch,
    util::{get_ip_address, is_ec2b_base64},
};
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
    pub fn create_from_simple_message(
        proto_dec_result: DecodingResult,
        dispatch: Dispatch,
    ) -> (Self, String) {
        let mut hotfix = Hotfix::default();
        let mut proto_body = String::from("\n");

        let mut unk_idx = 1;
        for field in &proto_dec_result.fields {
            let mut field_content = String::with_capacity(0);
            match field.wire_type {
                WireType::VarInt => {
                    // We try to find bool that set to "true". Bool represented as varint with value of 1.
                    // We also try to find port, it will be varint other than 1
                    if let DecodedValue::BigInt(num) = field.value {
                        if num == 1 {
                            field_content = format!("\tbool unk{unk_idx} = {};\n", field.field);
                            unk_idx += 1;
                            // Ensure value is within valid port range
                        } else if (23301..=23302).contains(&num) {
                            field_content = format!("\tuint32 port = {};\n", field.field);
                        }
                    }
                }
                WireType::Len => {
                    let DecodedValue::Buffer(buffer) = &field.value else {
                        continue;
                    };

                    // We try to find the dispatch urls as well as other string fields
                    if let Ok(v) = String::from_utf8(buffer.to_vec()) {
                        let field_name = match v {
                            v if v.contains("/asb/") => {
                                hotfix.asset_bundle_url = v;
                                "asset_bundle_url"
                            }
                            v if v.contains("/design_data/") => {
                                hotfix.ex_resource_url = v;
                                "ex_resource_url"
                            }
                            v if v.contains("/lua/") => {
                                hotfix.lua_url = v;
                                "lua_url"
                            }
                            v if v.contains("/ifix/") => {
                                hotfix.ifix_url = v;
                                "ifix_url"
                            }
                            v if v.contains("Access verification") => "msg",
                            v if v.eq(&dispatch.region_list[0].name) => "region_name",
                            v if get_ip_address(&v).is_some() => "ip",
                            v if is_ec2b_base64(&v) => "client_secret_key",
                            _ => "",
                        };

                        if !field_name.is_empty() {
                            field_content = format!("\tstring {} = {};\n", field_name, field.field);
                        }
                    }
                }
                _ => {}
            }
            proto_body += &field_content;
        }

        // We still have 2 fields left, mdk_res_version (lua_version) and ifix_version, we try to get that from the link we got before
        let lua_version = hotfix
            .lua_url
            .rsplit('/')
            .next()
            .unwrap_or_default()
            .split('_')
            .nth(1)
            .unwrap_or_default();

        let ifix_version = hotfix
            .ifix_url
            .rsplit('/')
            .next()
            .unwrap_or_default()
            .split('_')
            .nth(1)
            .unwrap_or_default();

        for field in proto_dec_result.fields {
            let mut field_content = String::with_capacity(0);
            if field.wire_type != WireType::Len {
                continue;
            }

            let DecodedValue::Buffer(buf) = field.value else {
                continue;
            };

            if let Ok(v) = String::from_utf8(buf) {
                let field_name = match v {
                    v if v == lua_version => "mdk_res_version",
                    v if v == ifix_version => "ifix_version",
                    _ => "",
                };

                if !field_name.is_empty() {
                    field_content = format!("\tstring {} = {};\n", field_name, field.field);
                }
            }

            proto_body += &field_content;
        }

        (
            hotfix,
            format!("syntax = \"proto3\";\n\nmessage Gateserver {{{proto_body}}}"),
        )
    }
}
