use prost::Message;

use crate::proto::{
    ChangeLineupLeaderCsReq, ChangeLineupLeaderScRsp, ExtraLineupType, GetAllLineupDataScRsp,
    GetCurLineupDataScRsp, LineupInfo,
};

use super::{lineup_avatar_infos, GameServerState};

fn build_lineup(state: &GameServerState) -> LineupInfo {
    let avatars = lineup_avatar_infos(&state.data);
    let leader_slot = if avatars.is_empty() { 0 } else { 0 };

    LineupInfo {
        is_virtual: false,
        plane_id: 20313,
        name: "Robin Team".to_string(),
        mp: 0,
        max_mp: 5,
        index: 0,
        extra_lineup_type: ExtraLineupType::LineupNone as i32,
        avatar_list: avatars,
        leader_slot,
        ..Default::default()
    }
}

pub fn on_change_lineup_leader(body: &[u8]) -> ChangeLineupLeaderScRsp {
    let slot = ChangeLineupLeaderCsReq::decode(body)
        .map(|v| v.slot)
        .unwrap_or(0);

    ChangeLineupLeaderScRsp { retcode: 0, slot }
}

pub fn on_get_all_lineup_data(state: &GameServerState) -> GetAllLineupDataScRsp {
    GetAllLineupDataScRsp {
        retcode: 0,
        lineup_list: vec![build_lineup(state)],
        cur_index: 0,
        ..Default::default()
    }
}

pub fn on_get_cur_lineup_data(state: &GameServerState) -> GetCurLineupDataScRsp {
    GetCurLineupDataScRsp {
        lineup: Some(build_lineup(state)),
        retcode: 0,
        ..Default::default()
    }
}
