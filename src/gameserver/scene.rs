use crate::proto::{
    scene_entity_info, AvatarType, GetCurSceneInfoScRsp, MotionInfo, SceneActorInfo,
    SceneEntityGroupInfo, SceneEntityInfo, SceneIdentifier, SceneInfo, ScenePropInfo, Vector,
};

use super::GameServerState;

pub fn on_get_cur_scene_info(state: &GameServerState) -> GetCurSceneInfoScRsp {
    let entity_id = 1;
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
            plane_id: 20503,
            entry_id: 2050301,
            floor_id: 20503001,
            world_id: 601,
            game_mode_type: 1,
            leader_entity_id: entity_id,
            scene_identifier: Some(SceneIdentifier {
                floor_id: 20503001,
                ..Default::default()
            }),
            entity_group_list: vec![
                SceneEntityGroupInfo {
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
                                x: 49087,
                                y: 9648,
                                z: 136907,
                            }),
                            rot: Some(Vector { x: 0, y: 0, z: 0 }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                SceneEntityGroupInfo {
                    group_id: 20,
                    state: 1,
                    entity_list: vec![SceneEntityInfo {
                        entity_id: 84,
                        group_id: 239,
                        inst_id: 300001,
                        entity: Some(scene_entity_info::Entity::Prop(ScenePropInfo {
                            prop_id: 808,
                            prop_state: 8,
                            ..Default::default()
                        })),
                        motion: Some(MotionInfo {
                            pos: Some(Vector {
                                x: 48730,
                                y: 9648,
                                z: 138200,
                            }),
                            rot: Some(Vector { x: 0, y: 0, z: 0 }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            ],
            ..Default::default()
        }),
        retcode: 0,
        ..Default::default()
    }
}
