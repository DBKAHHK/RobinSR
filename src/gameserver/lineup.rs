use std::collections::HashSet;

use prost::Message;

use crate::proto::{
    ChangeLineupLeaderCsReq, ChangeLineupLeaderScRsp, ExtraLineupType, GetAllLineupDataScRsp,
    GetCurLineupDataScRsp, LineupInfo, ReplaceLineupCsReq, ReplaceLineupScRsp, SyncLineupNotify,
    SyncLineupReason,
};

use super::{lineup_avatar_infos, persist_runtime, GameServerState};

fn build_lineup(state: &GameServerState) -> LineupInfo {
    let avatars = lineup_avatar_infos(state);
    let leader_slot = {
        let guard = state.runtime.read().expect("runtime read");
        guard.leader_slot
    };

    LineupInfo {
        is_virtual: false,
        plane_id: 20503,
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

pub fn build_sync_lineup_notify(state: &GameServerState) -> SyncLineupNotify {
    SyncLineupNotify {
        lineup: Some(build_lineup(state)),
        reason_list: vec![SyncLineupReason::SyncReasonNone as i32],
    }
}

pub fn on_change_lineup_leader(state: &GameServerState, body: &[u8]) -> ChangeLineupLeaderScRsp {
    let req_slot = ChangeLineupLeaderCsReq::decode(body)
        .map(|v| v.slot)
        .unwrap_or(0);
    let mut slot = req_slot;
    if let Ok(mut guard) = state.runtime.write() {
        let max_slot = guard.lineup.len().saturating_sub(1) as u32;
        slot = slot.min(max_slot);
        guard.leader_slot = slot;
    }
    if let Err(e) = persist_runtime(state) {
        eprintln!("failed to persist runtime after leader change: {e}");
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
    let existing_avatar_ids: HashSet<u32> = state.data.avatars.iter().map(|a| a.avatar_id).collect();
    let mut slots = req.lineup_slot_list;
    slots.sort_by_key(|s| s.slot);

    let mut seen = HashSet::new();
    let mut new_lineup = Vec::with_capacity(4);
    for s in slots {
        if s.id == 0 || !existing_avatar_ids.contains(&s.id) || !seen.insert(s.id) {
            continue;
        }
        new_lineup.push(s.id);
        if new_lineup.len() >= 4 {
            break;
        }
    }

    if let Ok(mut guard) = state.runtime.write() {
        if !new_lineup.is_empty() {
            guard.lineup = new_lineup;
        }
        let max_slot = guard.lineup.len().saturating_sub(1) as u32;
        guard.leader_slot = req.leader_slot.min(max_slot);
    }
    if let Err(e) = persist_runtime(state) {
        eprintln!("failed to persist runtime after replace lineup: {e}");
    }

    ReplaceLineupScRsp {
        retcode: 0,
        ..Default::default()
    }
}
