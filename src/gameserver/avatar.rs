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
            cur_multi_path_avatar_type: a.avatar_id,
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
