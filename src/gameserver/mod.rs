use std::{collections::HashMap, io, sync::Arc};

use prost::Message;
use tokio::net::TcpListener;

use crate::{data::GameData, packet::{Connection, Packet}, proto::{self, PlayerLoginFinishScRsp}};

mod avatar;
mod battle;
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
    get_cur_scene_info_cs_req: u16,
    get_cur_scene_info_sc_rsp: u16,
    get_mission_status_cs_req: u16,
    get_mission_status_sc_rsp: u16,
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
struct GameServerState {
    data: Arc<GameData>,
    cmd: Arc<CmdTable>,
}

pub async fn start(data: Arc<GameData>) -> io::Result<()> {
    let cmd = Arc::new(build_cmd_table());
    let state = GameServerState {
        data,
        cmd,
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
    let h = &state.cmd.handled;
    match pkt.cmd {
        cmd if cmd == h.player_get_token_cs_req => {
            let rsp = player::on_player_get_token(state);
            conn.send_raw(h.player_get_token_sc_rsp, &encode_msg(&rsp)).await
        }
        cmd if cmd == h.player_heart_beat_cs_req => {
            let rsp = player::on_player_heart_beat(&pkt.body);
            conn.send_raw(h.player_heart_beat_sc_rsp, &encode_msg(&rsp)).await
        }
        cmd if cmd == h.player_login_cs_req => {
            let rsp = player::on_player_login(state);
            conn.send_raw(h.player_login_sc_rsp, &encode_msg(&rsp)).await
        }
        cmd if cmd == h.get_basic_info_cs_req => {
            let rsp = player::on_get_basic_info();
            conn.send_raw(h.get_basic_info_sc_rsp, &encode_msg(&rsp)).await
        }
        cmd if cmd == h.get_player_board_data_cs_req => {
            let rsp = player::on_get_player_board_data(state);
            conn.send_raw(h.get_player_board_data_sc_rsp, &encode_msg(&rsp)).await
        }
        cmd if cmd == h.get_avatar_data_cs_req => {
            let rsp = avatar::on_get_avatar_data(state);
            conn.send_raw(h.get_avatar_data_sc_rsp, &encode_msg(&rsp)).await
        }
        cmd if cmd == h.get_cur_battle_info_cs_req => {
            let rsp = battle::on_get_cur_battle_info();
            conn.send_raw(h.get_cur_battle_info_sc_rsp, &encode_msg(&rsp)).await
        }
        cmd if cmd == h.get_all_lineup_data_cs_req => {
            let rsp = lineup::on_get_all_lineup_data(state);
            conn.send_raw(h.get_all_lineup_data_sc_rsp, &encode_msg(&rsp)).await
        }
        cmd if cmd == h.get_cur_lineup_data_cs_req => {
            let rsp = lineup::on_get_cur_lineup_data(state);
            conn.send_raw(h.get_cur_lineup_data_sc_rsp, &encode_msg(&rsp)).await
        }
        cmd if cmd == h.change_lineup_leader_cs_req => {
            let rsp = lineup::on_change_lineup_leader(&pkt.body);
            conn.send_raw(h.change_lineup_leader_sc_rsp, &encode_msg(&rsp)).await
        }
        cmd if cmd == h.get_cur_scene_info_cs_req => {
            let rsp = scene::on_get_cur_scene_info(state);
            conn.send_raw(h.get_cur_scene_info_sc_rsp, &encode_msg(&rsp)).await
        }
        cmd if cmd == h.get_mission_status_cs_req => {
            let rsp = mission::on_get_mission_status(&pkt.body);
            conn.send_raw(h.get_mission_status_sc_rsp, &encode_msg(&rsp)).await
        }
        cmd if cmd == h.player_login_finish_cs_req => {
            let rsp = PlayerLoginFinishScRsp { retcode: 0 };
            conn.send_raw(h.player_login_finish_sc_rsp, &encode_msg(&rsp)).await
        }
        _ => {
            if let Some(rsp_cmd) = state.cmd.dummy_map.get(&pkt.cmd) {
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
        get_cur_scene_info_cs_req: cmd_id(&name_to_id, "GetCurSceneInfoCsReq"),
        get_cur_scene_info_sc_rsp: cmd_id(&name_to_id, "GetCurSceneInfoScRsp"),
        get_mission_status_cs_req: cmd_id(&name_to_id, "GetMissionStatusCsReq"),
        get_mission_status_sc_rsp: cmd_id(&name_to_id, "GetMissionStatusScRsp"),
        player_login_finish_cs_req: cmd_id(&name_to_id, "PlayerLoginFinishCsReq"),
        player_login_finish_sc_rsp: cmd_id(&name_to_id, "PlayerLoginFinishScRsp"),
    };

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
        handled.player_login_finish_cs_req,
    ] {
        dummy_map.remove(&handled);
    }

    let name_by_id = name_to_id
        .iter()
        .map(|(name, id)| (*id, name.clone()))
        .collect();

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

fn default_lineup(data: &GameData) -> Vec<u32> {
    if !data.lineup.is_empty() {
        return data.lineup.clone();
    }
    data.avatars.iter().map(|a| a.avatar_id).take(4).collect()
}

fn lineup_avatar_infos(data: &GameData) -> Vec<proto::LineupAvatar> {
    default_lineup(data)
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
