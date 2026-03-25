use prost::Message;

use crate::proto::{
    ChangeLineupLeaderCsReq, ChangeLineupLeaderScRsp, ExtraLineupType, GetAllLineupDataScRsp,
    GetCurLineupDataScRsp, LineupInfo, ReplaceLineupCsReq, ReplaceLineupScRsp,
};

use super::{lineup_avatar_infos, GameServerState};

fn build_lineup(state: &GameServerState) -> LineupInfo {
    let avatars = lineup_avatar_infos(state);
    let leader_slot = {
        let guard = state.runtime.read().expect("runtime read");
        guard.leader_slot
    };

    LineupInfo {
        is_virtual: false,
        plane_id: 20313,
        name: "Robin Team".to_string(),
        mp: 5,
        max_mp: 5,
        index: 0,
        extra_lineup_type: ExtraLineupType::LineupNone as i32,
        avatar_list: avatars,
        leader_slot,
        ..Default::default()
    }
}

pub fn on_change_lineup_leader(state: &GameServerState, body: &[u8]) -> ChangeLineupLeaderScRsp {
    let slot = ChangeLineupLeaderCsReq::decode(body)
        .map(|v| v.slot)
        .unwrap_or(0);
    if let Ok(mut guard) = state.runtime.write() {
        guard.leader_slot = slot;
    }

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

pub fn on_replace_lineup(state: &GameServerState, body: &[u8]) -> ReplaceLineupScRsp {
    let req = ReplaceLineupCsReq::decode(body).unwrap_or_default();
    let mut slots = req.lineup_slot_list;
    slots.sort_by_key(|s| s.slot);
    let new_lineup: Vec<u32> = slots.into_iter().map(|s| s.id).filter(|id| *id != 0).collect();

    if let Ok(mut guard) = state.runtime.write() {
        if !new_lineup.is_empty() {
            guard.lineup = new_lineup;
        }
        guard.leader_slot = req.leader_slot;
    }

    ReplaceLineupScRsp {
        retcode: 0,
        ..Default::default()
    }
}
