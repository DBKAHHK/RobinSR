use std::{net::SocketAddr, sync::Arc};

use axum::{
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use base64::{engine::general_purpose, Engine as _};
use prost::Message;
use serde_json::json;

use crate::{data::GameData, proto::{Dispatch, GateServer, RegionInfo}};

const SDK_HOST: &str = "127.0.0.1";
const SDK_PORT: u16 = 21000;
const GAME_HOST: &str = "127.0.0.1";
const GAME_PORT: u16 = 23301;

pub async fn start(game_data: Arc<GameData>) -> std::io::Result<()> {
    let app = Router::new()
        .route("/query_dispatch", get(query_dispatch))
        .route("/query_gateway", get(query_gateway))
        .route("/hkrpg_cn/mdk/shield/api/login", post(login))
        .route("/hkrpg_cn/mdk/shield/api/verify", post(login))
        .route("/hkrpg_cn/combo/granter/login/v2/login", post(combo_login))
        .route("/account/risky/api/check", post(risk_check))
        .route("/account/ma-cn-passport/app/loginByPassword", post(apn_login))
        .route("/account/ma-cn-session/app/verify", post(apn_verify))
        .with_state(game_data);

    let addr = SocketAddr::from(([127, 0, 0, 1], SDK_PORT));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("sdkserver listening on {SDK_HOST}:{SDK_PORT}");
    axum::serve(listener, app)
        .await
        .map_err(std::io::Error::other)
}

async fn query_dispatch() -> impl IntoResponse {
    let rsp = Dispatch {
        retcode: 0,
        msg: "OK".to_string(),
        region_list: vec![RegionInfo {
            name: "prod_gf_cn".to_string(),
            title: "RobinSR".to_string(),
            dispatch_url: format!("http://{SDK_HOST}:{SDK_PORT}/query_gateway"),
            env_type: "9".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let mut buf = Vec::new();
    rsp.encode(&mut buf).expect("dispatch encode");
    general_purpose::STANDARD.encode(buf)
}

async fn query_gateway() -> impl IntoResponse {
    let rsp = GateServer {
        ip: GAME_HOST.to_string(),
        lua_url: "".to_string(),
        port: GAME_PORT as u32,
        region_name: "prod_gf_cn".to_string(),
        unk1: true,
        ex_resource_url: "".to_string(),
        asset_bundle_url: "".to_string(),
        retcode: 0,
        unk2: true,
        unk3: true,
        unk4: true,
        unk5: true,
        unk6: true,
        unk7: true,
        unk8: true,
        unk9: true,
        unk10: true,
        unk11: true,
        unk12: true,
        unk13: true,
        unk14: true,
        unk15: true,
        ifix_version: "0".to_string(),
        ..Default::default()
    };

    let mut buf = Vec::new();
    rsp.encode(&mut buf).expect("gate encode");
    general_purpose::STANDARD.encode(buf)
}

async fn login(axum::extract::State(game_data): axum::extract::State<Arc<GameData>>) -> Json<serde_json::Value> {
    Json(json!({
        "data": {
            "account": {
                "area_code": "**",
                "email": "robin@robinsr.local",
                "country": "US",
                "is_email_verify": "1",
                "token": game_data.token,
                "uid": game_data.uid.to_string(),
            },
            "device_grant_required": false,
            "reactivate_required": false,
            "realperson_required": false,
            "safe_mobile_required": false,
        },
        "message": "OK",
        "retcode": 0,
    }))
}

async fn combo_login(axum::extract::State(game_data): axum::extract::State<Arc<GameData>>) -> Json<serde_json::Value> {
    Json(json!({
        "data": {
            "account_type": 1,
            "combo_id": game_data.uid.to_string(),
            "combo_token": game_data.token,
            "data": "{\"guest\":false}",
            "heartbeat": false,
            "open_id": game_data.uid.to_string(),
        },
        "message": "OK",
        "retcode": 0,
    }))
}

async fn risk_check(axum::extract::State(game_data): axum::extract::State<Arc<GameData>>) -> Json<serde_json::Value> {
    Json(json!({
        "data": {"id": game_data.token, "action": "ACTION_NONE", "geetest": null},
        "message": "OK",
        "retcode": 0,
    }))
}

async fn apn_login(axum::extract::State(game_data): axum::extract::State<Arc<GameData>>) -> Json<serde_json::Value> {
    Json(json!({
        "data": {
            "token": {"token": game_data.token, "token_type": 1},
            "user_info": {
                "aid": game_data.uid.to_string(),
                "mid": game_data.uid.to_string(),
                "is_email_verify": 1,
                "area_code": "**",
                "country": "US",
                "is_adult": 1,
                "email": "robin@robinsr.local"
            }
        },
        "message": "OK",
        "retcode": 0
    }))
}

async fn apn_verify(axum::extract::State(game_data): axum::extract::State<Arc<GameData>>) -> Json<serde_json::Value> {
    Json(json!({
        "data": {
            "tokens": [{"token": game_data.token, "token_type": 1}],
            "user_info": {
                "aid": game_data.uid.to_string(),
                "mid": game_data.uid.to_string(),
                "is_email_verify": 1,
                "area_code": "**",
                "country": "US",
                "is_adult": 1,
                "email": "robin@robinsr.local"
            }
        },
        "message": "OK",
        "retcode": 0
    }))
}
