mod data;
mod gameserver;
mod packet;
mod proto;
mod sdkserver;

use std::sync::Arc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let data = Arc::new(data::load_data("freesr-data.json", "persistent.json")?);

    let sdk_data = Arc::clone(&data);
    let game_data = Arc::clone(&data);

    let sdk = tokio::spawn(async move { sdkserver::start(sdk_data).await });
    let game = tokio::spawn(async move { gameserver::start(game_data).await });

    tokio::select! {
        r = sdk => r.map_err(std::io::Error::other)?,
        r = game => r.map_err(std::io::Error::other)?,
    }
}
