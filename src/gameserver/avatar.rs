use prost::Message;
use std::collections::HashMap;

use crate::proto::{
    Avatar, AvatarPathData, AvatarPathSkillTree, EquipRelic, GetAvatarDataCsReq, GetAvatarDataScRsp,
    Gkdekjkoijn, SetAvatarPathCsReq, SetAvatarPathScRsp,
};

use super::{now_ms, GameServerState};

const MULTI_PATH_IDS: &[u32] = &[0, 1001, 1224, 8001, 8002, 8003, 8004, 8005, 8006, 8007, 8008, 8009, 8010];
const TAKEN_PROMOTION_REWARDS: [u32; 6] = [1, 2, 3, 4, 5, 6];
const SKIN_LIST: [u32; 2] = [1100101, 1131001];

pub fn on_get_avatar_data(state: &GameServerState, body: &[u8]) -> GetAvatarDataScRsp {
    let req = GetAvatarDataCsReq::decode(body).unwrap_or_default();
    let ts = now_ms();
    let runtime_paths = runtime_multi_path_types(state);

    let lightcone_by_avatar: HashMap<u32, u32> = state
        .data
        .lightcones
        .iter()
        .map(|lc| (lc.equip_avatar, lc.internal_uid + 2000))
        .collect();

    let avatar_path_data_info_list = state
        .data
        .avatars
        .iter()
        .map(|avatar| build_avatar_path_data(avatar, ts, &lightcone_by_avatar, state))
        .collect();

    let avatar_list = state
        .data
        .avatars
        .iter()
        .map(|avatar| build_avatar(avatar, &lightcone_by_avatar, runtime_paths))
        .collect();

    let mmekfjdmilj = MULTI_PATH_IDS
        .iter()
        .filter_map(|avatar_id| {
            state
                .data
                .avatars
                .iter()
                .find(|a| a.avatar_id == *avatar_id)
                .map(|a| Gkdekjkoijn {
                    key: *avatar_id,
                    value: a.rank,
                })
        })
        .collect();

    GetAvatarDataScRsp {
        is_get_all: req.is_get_all,
        retcode: 0,
        basic_type_id_list: MULTI_PATH_IDS.to_vec(),
        avatar_path_data_info_list,
        avatar_list,
        mmekfjdmilj,
        skin_list: SKIN_LIST.to_vec(),
        ..Default::default()
    }
}

pub fn on_set_avatar_path(state: &GameServerState, body: &[u8]) -> SetAvatarPathScRsp {
    let req = SetAvatarPathCsReq::decode(body).unwrap_or_default();
    let avatar_id = req.avatar_id as u32;

    if let Ok(mut guard) = state.runtime.write() {
        if is_trailblazer_avatar(avatar_id) {
            guard.mc_id = avatar_id;
        } else if is_march_avatar(avatar_id) {
            guard.march_id = avatar_id;
        }
    }

    SetAvatarPathScRsp {
        retcode: 0,
        avatar_id: req.avatar_id,
        ..Default::default()
    }
}

fn build_avatar(
    avatar: &crate::data::AvatarRecord,
    lightcone_by_avatar: &HashMap<u32, u32>,
    runtime_paths: (u32, u32),
) -> Avatar {
    let base_avatar_id = canonical_base_avatar_id(avatar.avatar_id);
    Avatar {
        first_met_time_stamp: 0,
        level: avatar.level,
        promotion: avatar.promotion,
        base_avatar_id,
        cur_multi_path_avatar_type: resolve_multi_path_avatar_type(avatar.avatar_id, runtime_paths),
        equipment_unique_id: *lightcone_by_avatar.get(&base_avatar_id).unwrap_or(&0),
        has_taken_promotion_reward_list: TAKEN_PROMOTION_REWARDS.to_vec(),
        ..Default::default()
    }
}

fn build_avatar_path_data(
    avatar: &crate::data::AvatarRecord,
    ts: u64,
    lightcone_by_avatar: &HashMap<u32, u32>,
    state: &GameServerState,
) -> AvatarPathData {
    let base_avatar_id = canonical_base_avatar_id(avatar.avatar_id);
    AvatarPathData {
        avatar_path_skill_tree: avatar
            .skills_by_anchor_type
            .iter()
            .map(|(point_id, level)| AvatarPathSkillTree {
                point_id: *point_id,
                level: *level,
            })
            .collect(),
        avatar_id: base_avatar_id,
        rank: avatar.rank,
        unlock_timestamp: ts,
        path_equipment_id: *lightcone_by_avatar.get(&base_avatar_id).unwrap_or(&0),
        equip_relic_list: state
            .data
            .relics
            .iter()
            .filter(|r| r.equip_avatar == base_avatar_id)
            .map(|r| EquipRelic {
                r#type: r.relic_id % 10,
                relic_unique_id: r.internal_uid + 1,
            })
            .collect(),
        dressed_skin_id: dressed_skin_id_for(base_avatar_id),
        unk_enhanced_id: avatar.enhanced_id,
        ..Default::default()
    }
}

fn runtime_multi_path_types(state: &GameServerState) -> (u32, u32) {
    let (mc_id, march_id) = {
        let guard = state.runtime.read().expect("runtime read");
        (guard.mc_id, guard.march_id)
    };
    (mc_id, march_id)
}

fn resolve_multi_path_avatar_type(avatar_id: u32, runtime_paths: (u32, u32)) -> u32 {
    let (mc_id, march_id) = runtime_paths;
    if is_trailblazer_avatar(avatar_id) {
        return mc_id;
    }
    if is_march_avatar(avatar_id) {
        return march_id;
    }
    avatar_id
}

fn canonical_base_avatar_id(avatar_id: u32) -> u32 {
    if is_trailblazer_avatar(avatar_id) {
        8001
    } else if is_march_avatar(avatar_id) {
        1001
    } else {
        avatar_id
    }
}

fn dressed_skin_id_for(avatar_id: u32) -> u32 {
    match avatar_id {
        1001 => 1100101,
        1310 => 1131001,
        _ => 0,
    }
}

fn is_trailblazer_avatar(avatar_id: u32) -> bool {
    matches!(
        avatar_id,
        8001 | 8002 | 8003 | 8004 | 8005 | 8006 | 8007 | 8008 | 8009 | 8010
    )
}

fn is_march_avatar(avatar_id: u32) -> bool {
    matches!(avatar_id, 1001 | 1224)
}
