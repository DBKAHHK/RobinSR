use std::{collections::HashMap, io, sync::Arc};

use prost::Message;
use serde_json::json;
use tokio::net::TcpListener;

use crate::{data::GameData, packet::{Connection, Packet}, proto::{self, PlayerLoginFinishScRsp}};

mod avatar;
mod battle;
mod item;
mod lineup;
mod mission;
mod player;
mod scene;

const GAME_HOST: &str = "127.0.0.1";
const GAME_PORT: u16 = 23301;

#[derive(Clone)]
struct HandledCmds {
    player_get_token_cs_req: u16,
    player_get_token_sc_rsp: u16,
    player_heart_beat_cs_req: u16,
    player_heart_beat_sc_rsp: u16,
    player_login_cs_req: u16,
    player_login_sc_rsp: u16,
    get_basic_info_cs_req: u16,
    get_basic_info_sc_rsp: u16,
    get_player_board_data_cs_req: u16,
    get_player_board_data_sc_rsp: u16,
    get_avatar_data_cs_req: u16,
    get_avatar_data_sc_rsp: u16,
    get_cur_battle_info_cs_req: u16,
    get_cur_battle_info_sc_rsp: u16,
    get_all_lineup_data_cs_req: u16,
    get_all_lineup_data_sc_rsp: u16,
    get_cur_lineup_data_cs_req: u16,
    get_cur_lineup_data_sc_rsp: u16,
    change_lineup_leader_cs_req: u16,
    change_lineup_leader_sc_rsp: u16,
    sync_lineup_notify: u16,
    get_cur_scene_info_cs_req: u16,
    get_cur_scene_info_sc_rsp: u16,
    get_mission_status_cs_req: u16,
    get_mission_status_sc_rsp: u16,
    content_package_get_data_cs_req: u16,
    content_package_get_data_sc_rsp: u16,
    start_cocoon_stage_cs_req: u16,
    start_cocoon_stage_sc_rsp: u16,
    pve_battle_result_cs_req: u16,
    pve_battle_result_sc_rsp: u16,
    scene_cast_skill_cs_req: u16,
    scene_cast_skill_sc_rsp: u16,
    scene_cast_skill_cost_mp_cs_req: u16,
    scene_cast_skill_cost_mp_sc_rsp: u16,
    sync_client_res_version_cs_req: u16,
    sync_client_res_version_sc_rsp: u16,
    get_bag_cs_req: u16,
    get_bag_sc_rsp: u16,
    player_logout_cs_req: u16,
    set_client_paused_cs_req: u16,
    set_client_paused_sc_rsp: u16,
    replace_lineup_cs_req: u16,
    replace_lineup_sc_rsp: u16,
    set_avatar_path_cs_req: u16,
    set_avatar_path_sc_rsp: u16,
    player_login_finish_cs_req: u16,
    player_login_finish_sc_rsp: u16,
}

#[derive(Clone)]
struct CmdTable {
    handled: HandledCmds,
    dummy_map: HashMap<u16, u16>,
    name_by_id: HashMap<u16, String>,
}

#[derive(Clone)]
struct RuntimeState {
    mc_id: u32,
    march_id: u32,
    lineup: Vec<u32>,
    leader_slot: u32,
    client_paused: bool,
    on_battle: bool,
    current_battle_info: Option<proto::SceneBattleInfo>,
    last_battle_end_status: i32,
    next_battle_id: u32,
    last_world_level: u32,
}

#[derive(Clone)]
struct GameServerState {
    data: Arc<GameData>,
    cmd: Arc<CmdTable>,
    runtime: Arc<std::sync::RwLock<RuntimeState>>,
}

pub async fn start(data: Arc<GameData>) -> io::Result<()> {
    let cmd = Arc::new(build_cmd_table());
    let runtime = Arc::new(std::sync::RwLock::new(RuntimeState {
        mc_id: data.mc_id,
        march_id: data.march_id,
        lineup: if data.lineup.is_empty() {
            data.avatars.iter().map(|a| a.avatar_id).take(4).collect()
        } else {
            data.lineup.clone()
        },
        leader_slot: 0,
        client_paused: false,
        on_battle: false,
        current_battle_info: None,
        last_battle_end_status: proto::BattleEndStatus::BattleEndQuit as i32,
        next_battle_id: 1,
        last_world_level: 6,
    }));
    let state = GameServerState {
        data,
        cmd,
        runtime,
    };

    let listener = TcpListener::bind((GAME_HOST, GAME_PORT)).await?;
    println!("gameserver listening on {GAME_HOST}:{GAME_PORT}");

    loop {
        let (stream, addr) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            let (reader, writer) = stream.into_split();
            let mut conn = Connection::new(reader, writer);
            loop {
                let pkt = match conn.read_packet().await {
                    Ok(p) => p,
                    Err(_) => break,
                };

                let r = handle_packet(&state, &mut conn, pkt).await;
                if r.is_err() {
                    break;
                }
            }
            println!("client disconnected: {addr}");
        });
    }
}

async fn handle_packet(state: &GameServerState, conn: &mut Connection, pkt: Packet) -> io::Result<()> {
    log_packet("RX", state, pkt.cmd, &pkt.head, &pkt.body);
    let h = &state.cmd.handled;
    match pkt.cmd {
        cmd if cmd == h.player_get_token_cs_req => {
            let rsp = player::on_player_get_token(state);
            send_logged(state, conn, h.player_get_token_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.player_heart_beat_cs_req => {
            let rsp = player::on_player_heart_beat(&pkt.body);
            send_logged(state, conn, h.player_heart_beat_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.player_login_cs_req => {
            let rsp = player::on_player_login(state, &pkt.body);
            send_logged(state, conn, h.player_login_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.get_basic_info_cs_req => {
            let rsp = player::on_get_basic_info();
            send_logged(state, conn, h.get_basic_info_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.get_player_board_data_cs_req => {
            let rsp = player::on_get_player_board_data(state);
            send_logged(state, conn, h.get_player_board_data_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.get_avatar_data_cs_req => {
            let rsp = avatar::on_get_avatar_data(state, &pkt.body);
            send_logged(state, conn, h.get_avatar_data_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.get_cur_battle_info_cs_req => {
            let rsp = battle::on_get_cur_battle_info(state);
            send_logged(state, conn, h.get_cur_battle_info_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.get_all_lineup_data_cs_req => {
            let rsp = lineup::on_get_all_lineup_data(state);
            send_logged(state, conn, h.get_all_lineup_data_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.get_cur_lineup_data_cs_req => {
            let rsp = lineup::on_get_cur_lineup_data(state);
            send_logged(state, conn, h.get_cur_lineup_data_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.change_lineup_leader_cs_req => {
            let rsp = lineup::on_change_lineup_leader(state, &pkt.body);
            send_logged(state, conn, h.change_lineup_leader_sc_rsp, encode_msg(&rsp)).await?;
            let notify = lineup::build_sync_lineup_notify(state);
            send_logged(state, conn, h.sync_lineup_notify, encode_msg(&notify)).await
        }
        cmd if cmd == h.get_cur_scene_info_cs_req => {
            let rsp = scene::on_get_cur_scene_info(state);
            send_logged(state, conn, h.get_cur_scene_info_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.get_mission_status_cs_req => {
            let rsp = mission::on_get_mission_status(&pkt.body);
            send_logged(state, conn, h.get_mission_status_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.content_package_get_data_cs_req => {
            let rsp = player::on_content_package_get_data();
            send_logged(state, conn, h.content_package_get_data_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.start_cocoon_stage_cs_req => {
            let rsp = battle::on_start_cocoon_stage(state, &pkt.body);
            send_logged(state, conn, h.start_cocoon_stage_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.pve_battle_result_cs_req => {
            let rsp = battle::on_pve_battle_result(state, &pkt.body);
            send_logged(state, conn, h.pve_battle_result_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.scene_cast_skill_cs_req => {
            let rsp = battle::on_scene_cast_skill(state, &pkt.body);
            send_logged(state, conn, h.scene_cast_skill_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.scene_cast_skill_cost_mp_cs_req => {
            let rsp = battle::on_scene_cast_skill_cost_mp(&pkt.body);
            send_logged(state, conn, h.scene_cast_skill_cost_mp_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.sync_client_res_version_cs_req => {
            let rsp = battle::on_sync_client_res_version(&pkt.body);
            send_logged(state, conn, h.sync_client_res_version_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.get_bag_cs_req => {
            let rsp = item::on_get_bag(state);
            send_logged(state, conn, h.get_bag_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.player_logout_cs_req => {
            conn.close().await?;
            Err(io::Error::new(io::ErrorKind::ConnectionAborted, "player logout"))
        }
        cmd if cmd == h.set_client_paused_cs_req => {
            let rsp = player::on_set_client_paused(state, &pkt.body);
            send_logged(state, conn, h.set_client_paused_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.replace_lineup_cs_req => {
            let rsp = lineup::on_replace_lineup(state, &pkt.body);
            send_logged(state, conn, h.replace_lineup_sc_rsp, encode_msg(&rsp)).await?;
            let notify = lineup::build_sync_lineup_notify(state);
            send_logged(state, conn, h.sync_lineup_notify, encode_msg(&notify)).await
        }
        cmd if cmd == h.set_avatar_path_cs_req => {
            let rsp = avatar::on_set_avatar_path(state, &pkt.body);
            send_logged(state, conn, h.set_avatar_path_sc_rsp, encode_msg(&rsp)).await
        }
        cmd if cmd == h.player_login_finish_cs_req => {
            let rsp = PlayerLoginFinishScRsp { retcode: 0 };
            send_logged(state, conn, h.player_login_finish_sc_rsp, encode_msg(&rsp)).await
        }
        _ => {
            if let Some(rsp_cmd) = state.cmd.dummy_map.get(&pkt.cmd) {
                log_packet("TX", state, *rsp_cmd, &[], &[]);
                conn.send_empty(*rsp_cmd).await
            } else {
                let name = state
                    .cmd
                    .name_by_id
                    .get(&pkt.cmd)
                    .map_or("UNKNOWN", String::as_str);
                println!("unhandled cmd: {} ({})", pkt.cmd, name);
                Ok(())
            }
        }
    }
}

fn encode_msg<M: Message>(msg: &M) -> Vec<u8> {
    let mut buf = Vec::new();
    msg.encode(&mut buf).expect("encode message");
    buf
}

async fn send_logged(
    state: &GameServerState,
    conn: &mut Connection,
    cmd: u16,
    body: Vec<u8>,
) -> io::Result<()> {
    log_packet("TX", state, cmd, &[], &body);
    conn.send_raw(cmd, &body).await
}

fn log_packet(direction: &str, state: &GameServerState, cmd: u16, head: &[u8], body: &[u8]) {
    let name = state
        .cmd
        .name_by_id
        .get(&cmd)
        .map_or("UNKNOWN", String::as_str);
    println!(
        "[{direction}] cmd={cmd} ({name}) head_len={} body_len={} body_hex={}",
        head.len(),
        body.len(),
        hex_encode(body),
    );
}

fn hex_encode(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

fn build_cmd_table() -> CmdTable {
    let mut name_to_id = HashMap::new();

    for line in include_str!("../proto/cmdid.rs").lines() {
        let trimmed = line.trim();
        let Some((name_part, id_part)) = trimmed.split_once('=') else {
            continue;
        };

        let name = name_part.trim();
        if !name.ends_with(';') && !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }

        let id_text = id_part.trim().trim_end_matches(';').trim();
        let Ok(id) = id_text.parse::<u16>() else {
            continue;
        };
        name_to_id.insert(name.to_string(), id);
    }

    let mut dummy_map: HashMap<u16, u16> = HashMap::new();
    for (name, req_id) in &name_to_id {
        for rsp_name in rsp_candidates(name) {
            if let Some(rsp_id) = name_to_id.get(&rsp_name) {
                dummy_map.insert(*req_id, *rsp_id);
                break;
            }
        }
    }

    let handled = HandledCmds {
        player_get_token_cs_req: cmd_id(&name_to_id, "PlayerGetTokenCsReq"),
        player_get_token_sc_rsp: cmd_id(&name_to_id, "PlayerGetTokenScRsp"),
        player_heart_beat_cs_req: cmd_id(&name_to_id, "PlayerHeartBeatCsReq"),
        player_heart_beat_sc_rsp: cmd_id(&name_to_id, "PlayerHeartBeatScRsp"),
        player_login_cs_req: cmd_id(&name_to_id, "PlayerLoginCsReq"),
        player_login_sc_rsp: cmd_id(&name_to_id, "PlayerLoginScRsp"),
        get_basic_info_cs_req: cmd_id(&name_to_id, "GetBasicInfoCsReq"),
        get_basic_info_sc_rsp: cmd_id(&name_to_id, "GetBasicInfoScRsp"),
        get_player_board_data_cs_req: cmd_id(&name_to_id, "GetPlayerBoardDataCsReq"),
        get_player_board_data_sc_rsp: cmd_id(&name_to_id, "GetPlayerBoardDataScRsp"),
        get_avatar_data_cs_req: cmd_id(&name_to_id, "GetAvatarDataCsReq"),
        get_avatar_data_sc_rsp: cmd_id(&name_to_id, "GetAvatarDataScRsp"),
        get_cur_battle_info_cs_req: cmd_id(&name_to_id, "GetCurBattleInfoCsReq"),
        get_cur_battle_info_sc_rsp: cmd_id(&name_to_id, "GetCurBattleInfoScRsp"),
        get_all_lineup_data_cs_req: cmd_id(&name_to_id, "GetAllLineupDataCsReq"),
        get_all_lineup_data_sc_rsp: cmd_id(&name_to_id, "GetAllLineupDataScRsp"),
        get_cur_lineup_data_cs_req: cmd_id(&name_to_id, "GetCurLineupDataCsReq"),
        get_cur_lineup_data_sc_rsp: cmd_id(&name_to_id, "GetCurLineupDataScRsp"),
        change_lineup_leader_cs_req: cmd_id(&name_to_id, "ChangeLineupLeaderCsReq"),
        change_lineup_leader_sc_rsp: cmd_id(&name_to_id, "ChangeLineupLeaderScRsp"),
        sync_lineup_notify: cmd_id(&name_to_id, "SyncLineupNotify"),
        get_cur_scene_info_cs_req: cmd_id(&name_to_id, "GetCurSceneInfoCsReq"),
        get_cur_scene_info_sc_rsp: cmd_id(&name_to_id, "GetCurSceneInfoScRsp"),
        get_mission_status_cs_req: cmd_id(&name_to_id, "GetMissionStatusCsReq"),
        get_mission_status_sc_rsp: cmd_id(&name_to_id, "GetMissionStatusScRsp"),
        content_package_get_data_cs_req: cmd_id(&name_to_id, "ContentPackageGetDataCsReq"),
        content_package_get_data_sc_rsp: cmd_id(&name_to_id, "ContentPackageGetDataScRsp"),
        start_cocoon_stage_cs_req: cmd_id(&name_to_id, "StartCocoonStageCsReq"),
        start_cocoon_stage_sc_rsp: cmd_id(&name_to_id, "StartCocoonStageScRsp"),
        pve_battle_result_cs_req: cmd_id(&name_to_id, "PVEBattleResultCsReq"),
        pve_battle_result_sc_rsp: cmd_id(&name_to_id, "PVEBattleResultScRsp"),
        scene_cast_skill_cs_req: cmd_id(&name_to_id, "SceneCastSkillCsReq"),
        scene_cast_skill_sc_rsp: cmd_id(&name_to_id, "SceneCastSkillScRsp"),
        scene_cast_skill_cost_mp_cs_req: cmd_id(&name_to_id, "SceneCastSkillCostMpCsReq"),
        scene_cast_skill_cost_mp_sc_rsp: cmd_id(&name_to_id, "SceneCastSkillCostMpScRsp"),
        sync_client_res_version_cs_req: cmd_id(&name_to_id, "SyncClientResVersionCsReq"),
        sync_client_res_version_sc_rsp: cmd_id(&name_to_id, "SyncClientResVersionScRsp"),
        get_bag_cs_req: cmd_id(&name_to_id, "GetBagCsReq"),
        get_bag_sc_rsp: cmd_id(&name_to_id, "GetBagScRsp"),
        player_logout_cs_req: cmd_id(&name_to_id, "PlayerLogoutCsReq"),
        set_client_paused_cs_req: cmd_id(&name_to_id, "SetClientPausedCsReq"),
        set_client_paused_sc_rsp: cmd_id(&name_to_id, "SetClientPausedScRsp"),
        replace_lineup_cs_req: cmd_id(&name_to_id, "ReplaceLineupCsReq"),
        replace_lineup_sc_rsp: cmd_id(&name_to_id, "ReplaceLineupScRsp"),
        set_avatar_path_cs_req: cmd_id(&name_to_id, "SetAvatarPathCsReq"),
        set_avatar_path_sc_rsp: cmd_id(&name_to_id, "SetAvatarPathScRsp"),
        player_login_finish_cs_req: cmd_id(&name_to_id, "PlayerLoginFinishCsReq"),
        player_login_finish_sc_rsp: cmd_id(&name_to_id, "PlayerLoginFinishScRsp"),
    };

    let name_by_id = name_to_id
        .iter()
        .map(|(name, id)| (*id, name.clone()))
        .collect();

    for handled in [
        handled.player_get_token_cs_req,
        handled.player_heart_beat_cs_req,
        handled.player_login_cs_req,
        handled.get_basic_info_cs_req,
        handled.get_player_board_data_cs_req,
        handled.get_avatar_data_cs_req,
        handled.get_cur_battle_info_cs_req,
        handled.get_all_lineup_data_cs_req,
        handled.get_cur_lineup_data_cs_req,
        handled.change_lineup_leader_cs_req,
        handled.get_cur_scene_info_cs_req,
        handled.get_mission_status_cs_req,
        handled.content_package_get_data_cs_req,
        handled.start_cocoon_stage_cs_req,
        handled.pve_battle_result_cs_req,
        handled.scene_cast_skill_cs_req,
        handled.scene_cast_skill_cost_mp_cs_req,
        handled.sync_client_res_version_cs_req,
        handled.get_bag_cs_req,
        handled.player_logout_cs_req,
        handled.set_client_paused_cs_req,
        handled.replace_lineup_cs_req,
        handled.set_avatar_path_cs_req,
        handled.player_login_finish_cs_req,
    ] {
        dummy_map.remove(&handled);
    }

    CmdTable {
        handled,
        dummy_map,
        name_by_id,
    }
}

fn cmd_id(name_to_id: &HashMap<String, u16>, name: &str) -> u16 {
    *name_to_id
        .get(name)
        .unwrap_or_else(|| panic!("missing cmd id: {name}"))
}

fn rsp_candidates(name: &str) -> Vec<String> {
    let mut out = Vec::with_capacity(3);
    if let Some(base) = name.strip_suffix("CsReq") {
        out.push(format!("{base}ScRsp"));
    }
    if let Some(base) = name.strip_suffix("CSReq") {
        out.push(format!("{base}ScRsp"));
    }
    if let Some(base) = name.strip_suffix("Req") {
        out.push(format!("{base}ScRsp"));
    }
    out
}

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn persist_runtime(state: &GameServerState) -> io::Result<()> {
    let (mc_id, march_id, lineup) = {
        let guard = state.runtime.read().expect("runtime read");
        (
            guard.mc_id,
            guard.march_id,
            guard.lineup.clone(),
        )
    };

    let payload = json!({
        "avatar": {
            "mc_id": mc_id.to_string(),
            "march_id": march_id.to_string(),
            "lineup": lineup,
        }
    });

    let data = serde_json::to_string_pretty(&payload)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("serialize persistent: {e}")))?;
    std::fs::write("persistent.json", data)
}

fn lineup_avatar_infos(state: &GameServerState) -> Vec<proto::LineupAvatar> {
    let mut lineup = {
        let guard = state.runtime.read().expect("runtime read");
        guard.lineup.clone()
    };
    if lineup.len() > 4 {
        lineup.truncate(4);
    }
    lineup
        .into_iter()
        .enumerate()
        .map(|(idx, avatar_id)| proto::LineupAvatar {
            hp: 10_000,
            id: avatar_id,
            slot: idx as u32,
            sp_bar: Some(proto::SpBarInfo {
                cur_sp: 0,
                max_sp: 10_000,
                ..Default::default()
            }),
            avatar_type: proto::AvatarType::AvatarFormalType as i32,
            ..Default::default()
        })
        .collect()
}
