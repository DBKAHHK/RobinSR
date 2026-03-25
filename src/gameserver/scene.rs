use crate::proto::{
    scene_entity_info, AvatarType, GetCurSceneInfoScRsp, MotionInfo, SceneActorInfo,
    SceneEntityGroupInfo, SceneEntityInfo, SceneInfo, Vector,
};

use super::GameServerState;

pub fn on_get_cur_scene_info(state: &GameServerState) -> GetCurSceneInfoScRsp {
    let entity_id = (state.data.uid << 3) + 1;
    let leader_avatar = {
        let guard = state.runtime.read().expect("runtime read");
        guard
            .lineup
            .get(guard.leader_slot as usize)
            .copied()
            .unwrap_or(guard.mc_id)
    };

    GetCurSceneInfoScRsp {
        scene: Some(SceneInfo {
            plane_id: 20313,
            entry_id: 2031301,
            floor_id: 20313001,
            game_mode_type: 1,
            leader_entity_id: entity_id,
            entity_group_list: vec![SceneEntityGroupInfo {
                state: 1,
                entity_list: vec![SceneEntityInfo {
                    entity_id,
                    entity: Some(scene_entity_info::Entity::Actor(SceneActorInfo {
                        base_avatar_id: leader_avatar,
                        avatar_type: AvatarType::AvatarFormalType as i32,
                        map_layer: 2,
                        uid: state.data.uid,
                        ..Default::default()
                    })),
                    motion: Some(MotionInfo {
                        pos: Some(Vector {
                            x: 40748,
                            y: 192819,
                            z: 439218,
                        }),
                        rot: Some(Vector { x: 0, y: 0, z: 0 }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }),
        retcode: 0,
        ..Default::default()
    }
}
