use prost::Message;
use reqwest::Client;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

mod proto;
use proto::Dispatch;
mod decode;
use decode::{Decoder, simplify};
mod util;
use util::{
    get_binary_version_path, get_client_config_path, get_dispatch_seed, get_last_buffer_start,
    last_index_of, read_string, read_uint24_be, select_folder, split_buffer, strip_empty_bytes,
};
mod hotfix;
use hotfix::Hotfix;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(folder_path) = select_folder() {
        let start_time = Instant::now();

        let binary_version_path = get_binary_version_path(&folder_path);

        let client_config_path = get_client_config_path(&folder_path);

        let binary_version_buffer = fs::read(&binary_version_path)?;

        let client_config_buffer = fs::read(&client_config_path)?;

        let client_config_buffer_parts = strip_empty_bytes(&client_config_buffer);

        let query_dispatch_base = read_string(last_index_of(&client_config_buffer_parts, 0x00), 0);

        let last_buffer_start = get_last_buffer_start(&binary_version_buffer);

        let last_buffer = &binary_version_buffer[last_buffer_start..];

        let buffer_splits = split_buffer(last_buffer, 0x00);

        let branch = read_string(&binary_version_buffer, 1);

        let revision = read_uint24_be(buffer_splits[0], 0);

        let time = read_string(buffer_splits[1], 0);

        let constructed_string = format!("{}-{}-{}", time, branch, revision);

        let (version_str, seed_str) = match get_dispatch_seed(&buffer_splits, &constructed_string) {
            Some(v) => v,
            None => { println!("->> Dispatch seed not found."); return Ok(()) }
        };

        println!("->> Dispatch Seed: {}", seed_str);

        let version_split: Vec<&str> = version_str.split('-').collect();

        let version = version_split.get(4).unwrap_or(&"").to_string();

        let build = version_split.get(5).unwrap_or(&"").to_string();

        println!("->> Version: {}", version);

        println!("->> Build: {}", build);

        let query_dispatch_url = format!(
            "{}?version={}&language_type=3&platform_type=3&channel_id=1&sub_channel_id=1&is_new_format=1",
            query_dispatch_base, version
        );

        println!("->> Dispatch URL: {}", query_dispatch_url);

        let client = Client::new();

        let query_dispatch_response = client.get(&query_dispatch_url).send().await?.text().await?;

        let dispatch_decoded_base64 = rbase64::decode(&query_dispatch_response)?;

        let dispatch_decoded_message = Dispatch::decode(&*dispatch_decoded_base64)?;

        if dispatch_decoded_message.region_list.is_empty() {
            println!("->> region_list is empty.");
            return Ok(());
        }

        let query_gateway_base = &dispatch_decoded_message.region_list[0].dispatch_url;

        let query_gateway_url = format!(
            "{}?version={}&platform_type=1&language_type=3&dispatch_seed={}&channel_id=1&sub_channel_id=1&is_need_url=1",
            query_gateway_base, version, seed_str
        );

        println!("->> Gateway URL: {}", query_gateway_url);

        let query_gateway_response = client.get(&query_gateway_url).send().await?.text().await?;

        let gateserver_decoded_base64 = rbase64::decode(&query_gateway_response)?;

        let mut decoder = Decoder::new(gateserver_decoded_base64);

        let gateserver_decoded_message = decoder.decode()?;

        let simplified_gateserver = simplify(gateserver_decoded_message);

        let hotfix_json = Hotfix::create_from_simple_message(simplified_gateserver);

        let pretty_json = serde_json::to_string_pretty(&hotfix_json)?;

        let output_path = Path::new("hotfix.json");

        let mut file = fs::File::create(output_path)?;

        file.write_all(pretty_json.as_bytes())?;

        println!("->> Finished writing hotfix.json");

        println!("->> Elapsed time: {}s", start_time.elapsed().as_secs_f32());

        Ok(())
    } else {
        println!("->> No folder selected.");
        Ok(())
    }
}
