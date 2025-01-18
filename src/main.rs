use prost::Message;
use reqwest::Client;
use std::io::Write;
use std::time::Instant;
use std::{fs, path::PathBuf};

mod proto;
use proto::Dispatch;
mod decode;
use decode::Decoder;
mod binary_version;
mod client_config;
mod hotfix;

use hotfix::Hotfix;
mod util;
use binary_version::BinaryVersionData;
use client_config::ClientStartupConfig;
use util::{get_binary_version_path, get_client_config_path, select_folder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(folder_path) = select_folder() {
        let start_time = Instant::now();

        let binary_version_path = get_binary_version_path(&folder_path);
        let client_config_path = get_client_config_path(&folder_path);

        let client_config_buffer = fs::read(&client_config_path)?;
        let client_config = ClientStartupConfig::try_from(client_config_buffer)?;

        let binary_version_buffer = fs::read(&binary_version_path)?;
        let binary_version = BinaryVersionData::try_from(binary_version_buffer)?;

        let game_version = binary_version
            .get_server_pak_type_version()
            .expect("cannot find game version!");

        println!("->> Version: {}", binary_version.version_string);
        println!("->> Build: {}", binary_version.branch);

        let query_dispatch_url = format!(
            "{}?version={}&language_type=3&platform_type=3&channel_id=1&sub_channel_id=1&is_new_format=1",
            client_config
                .global_dispatch_url_list
                .first()
                .expect("cannot found dispatch url!"),
            game_version
        );

        println!("->> Dispatch URL: {}", query_dispatch_url);

        let client = &Client::new();

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
            query_gateway_base, game_version, binary_version.dispatch_seed
        );

        println!("->> Gateway URL: {}", query_gateway_url);

        let query_gateway_response = client.get(&query_gateway_url).send().await?.text().await?;

        let gateserver_decoded_base64 = rbase64::decode(&query_gateway_response)?;

        let mut decoder = Decoder::new(gateserver_decoded_base64);

        let gateserver_decoded_message = decoder.decode()?;

        let simplified_gateserver = gateserver_decoded_message.simplify();

        let hotfix_json = Hotfix::create_from_simple_message(simplified_gateserver);

        let pretty_json = serde_json::to_string_pretty(&hotfix_json)?;

        let output_path = PathBuf::from(format!("hotfix-{}.json", game_version));

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
