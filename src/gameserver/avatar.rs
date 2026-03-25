use crate::proto::{Avatar, AvatarPathData, AvatarPathSkillTree, GetAvatarDataScRsp};

use super::{now_ms, GameServerState};

pub fn on_get_avatar_data(state: &GameServerState) -> GetAvatarDataScRsp {
    let ts = now_ms();

    let avatar_list = state
        .data
        .avatars
        .iter()
        .map(|a| Avatar {
            first_met_time_stamp: ts,
            level: a.level,
            promotion: a.promotion,
            base_avatar_id: a.avatar_id,
            cur_multi_path_avatar_type: resolve_multi_path_avatar_type(state, a.avatar_id),
            ..Default::default()
        })
        .collect();

    let avatar_path_data_info_list = state
        .data
        .avatars
        .iter()
        .map(|a| AvatarPathData {
            avatar_path_skill_tree: a
                .skills_by_anchor_type
                .iter()
                .map(|(point_id, level)| AvatarPathSkillTree {
                    point_id: *point_id,
                    level: *level,
                })
                .collect(),
            avatar_id: a.avatar_id,
            rank: a.rank,
            unlock_timestamp: ts,
            unk_enhanced_id: a.enhanced_id,
            ..Default::default()
        })
        .collect();

    GetAvatarDataScRsp {
        is_get_all: true,
        retcode: 0,
        basic_type_id_list: state.data.avatars.iter().map(|a| a.avatar_id).collect(),
        avatar_path_data_info_list,
        avatar_list,
        ..Default::default()
    }
}

fn resolve_multi_path_avatar_type(state: &GameServerState, avatar_id: u32) -> u32 {
    if is_trailblazer_avatar(avatar_id) {
        return state.data.mc_id;
    }
    if is_march_avatar(avatar_id) {
        return state.data.march_id;
    }
    avatar_id
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
