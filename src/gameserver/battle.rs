use std::collections::HashMap;

use prost::Message;

use crate::proto::{
    AvatarSkillTree, AvatarType, BattleAvatar, BattleBuff, BattleEndStatus, BattleEquipment,
    BattleEventBattleInfo, BattleEventProperty, BattleRelic, BattleTarget, BattleTargetList,
    GetCurBattleInfoScRsp, HitMonsterBattleInfo, MonsterBattleType, PveBattleResultCsReq,
    PveBattleResultScRsp, RelicAffix, SceneBattleInfo, SceneCastSkillCostMpCsReq,
    SceneCastSkillCostMpScRsp, SceneCastSkillCsReq, SceneCastSkillScRsp, SceneMonster,
    SceneMonsterWave, SceneMonsterWaveParam, SpBarInfo, StartCocoonStageCsReq, StartCocoonStageScRsp,
    SyncClientResVersionCsReq, SyncClientResVersionScRsp,
};

use super::GameServerState;

pub fn on_get_cur_battle_info(state: &GameServerState) -> GetCurBattleInfoScRsp {
    let (on_battle, battle_info, last_end_status) = {
        let guard = state.runtime.read().expect("runtime read");
        (
            guard.on_battle,
            guard.current_battle_info.clone(),
            guard.last_battle_end_status,
        )
    };

    GetCurBattleInfoScRsp {
        battle_info: if on_battle { battle_info } else { None },
        last_end_status: if on_battle {
            BattleEndStatus::BattleEndNone as i32
        } else {
            last_end_status
        },
        retcode: 0,
        ..Default::default()
    }
}

pub fn on_start_cocoon_stage(state: &GameServerState, body: &[u8]) -> StartCocoonStageScRsp {
    let req = StartCocoonStageCsReq::decode(body).unwrap_or_default();
    let world_level = if req.world_level == 0 { 6 } else { req.world_level };
    let technique_avatar_id = req.dlaceefjahb.as_ref().map(|t| t.phoglephgof).unwrap_or(0);
    let battle_id = allocate_battle_id(state, world_level);
    let battle_info = create_battle(state, battle_id, world_level, technique_avatar_id);

    if let Ok(mut guard) = state.runtime.write() {
        guard.on_battle = true;
        guard.last_battle_end_status = BattleEndStatus::BattleEndNone as i32;
        guard.current_battle_info = Some(battle_info.clone());
    }

    StartCocoonStageScRsp {
        wave: req.wave,
        prop_entity_id: req.prop_entity_id,
        cocoon_id: req.cocoon_id,
        challenge_cnt: 1,
        battle_info: Some(battle_info),
        retcode: 0,
        ..Default::default()
    }
}

pub fn on_scene_cast_skill(state: &GameServerState, body: &[u8]) -> SceneCastSkillScRsp {
    let req = SceneCastSkillCsReq::decode(body).unwrap_or_default();
    let target_ids = collect_target_monster_ids(&req);

    let monster_battle_info: Vec<HitMonsterBattleInfo> = target_ids
        .iter()
        .map(|entity_id| HitMonsterBattleInfo {
            target_monster_entity_id: *entity_id,
            monster_battle_type: get_battle_type(*entity_id, req.attacked_by_entity_id, req.skill_index)
                as i32,
        })
        .collect();

    let should_trigger = monster_battle_info
        .iter()
        .any(|v| v.monster_battle_type == MonsterBattleType::TriggerBattle as i32);

    let battle_info = if should_trigger {
        let world_level = {
            let guard = state.runtime.read().expect("runtime read");
            guard.last_world_level
        };
        let battle_id = allocate_battle_id(state, world_level);
        Some(create_battle(state, battle_id, world_level, 0))
    } else {
        None
    };

    if should_trigger {
        if let Ok(mut guard) = state.runtime.write() {
            guard.on_battle = true;
            guard.last_battle_end_status = BattleEndStatus::BattleEndNone as i32;
            guard.current_battle_info = battle_info.clone();
        }
    }

    SceneCastSkillScRsp {
        battle_info,
        monster_battle_info,
        cast_entity_id: req.cast_entity_id,
        retcode: 0,
    }
}

pub fn on_pve_battle_result(state: &GameServerState, body: &[u8]) -> PveBattleResultScRsp {
    let req = PveBattleResultCsReq::decode(body).unwrap_or_default();
    let world_level = {
        let guard = state.runtime.read().expect("runtime read");
        guard.last_world_level
    };

    if let Ok(mut guard) = state.runtime.write() {
        guard.on_battle = false;
        guard.current_battle_info = None;
        guard.last_battle_end_status = if req.end_status != 0 {
            req.end_status
        } else {
            BattleEndStatus::BattleEndWin as i32
        };
    }

    PveBattleResultScRsp {
        battle_id: req.battle_id,
        stage_id: if req.stage_id == 0 {
            state.data.battle.stage_id
        } else {
            req.stage_id
        },
        end_status: req.end_status,
        battle_avatar_list: build_battle_avatars(state, world_level, 0),
        retcode: 0,
        ..Default::default()
    }
}

pub fn on_scene_cast_skill_cost_mp(body: &[u8]) -> SceneCastSkillCostMpScRsp {
    let req = SceneCastSkillCostMpCsReq::decode(body).unwrap_or_default();
    SceneCastSkillCostMpScRsp {
        cast_entity_id: req.cast_entity_id,
        retcode: 0,
    }
}

pub fn on_sync_client_res_version(body: &[u8]) -> SyncClientResVersionScRsp {
    let req = SyncClientResVersionCsReq::decode(body).unwrap_or_default();
    SyncClientResVersionScRsp {
        client_res_version: req.client_res_version,
        retcode: 0,
    }
}

fn create_battle(
    state: &GameServerState,
    battle_id: u32,
    world_level: u32,
    technique_avatar_id: u32,
) -> SceneBattleInfo {
    let stage_id = state.data.battle.stage_id;
    let mut battle = SceneBattleInfo {
        battle_id,
        stage_id,
        logic_random_seed: now_seed(),
        rounds_limit: state.data.battle.cycle_count,
        monster_wave_length: state.data.battle.monsters.len() as u32,
        world_level,
        ..Default::default()
    };

    battle.battle_avatar_list = build_battle_avatars(state, world_level, technique_avatar_id);
    battle.monster_wave_list = build_monster_waves(state);
    battle.buff_list = build_stage_blessings(state);
    add_trigger_attack_buff(&mut battle, state);
    add_global_passive_buffs(&mut battle, state);
    add_battle_targets(&mut battle);
    add_su_resonance_event(&mut battle, state);

    battle
}

fn build_battle_avatars(
    state: &GameServerState,
    world_level: u32,
    technique_avatar_id: u32,
) -> Vec<BattleAvatar> {
    let lineup = {
        let guard = state.runtime.read().expect("runtime read");
        guard.lineup.clone()
    };

    lineup
        .into_iter()
        .enumerate()
        .filter_map(|(idx, avatar_id)| {
            let a = state.data.avatars.iter().find(|v| v.avatar_id == avatar_id)?;
            let max_sp = if a.sp_max == 0 { 10_000 } else { a.sp_max };
            let cur_sp = if technique_avatar_id != 0 && technique_avatar_id == a.avatar_id {
                max_sp
            } else {
                a.sp_value.min(max_sp)
            };

            let equipment_list = state
                .data
                .lightcones
                .iter()
                .find(|lc| lc.equip_avatar == avatar_id)
                .map(|lc| {
                    vec![BattleEquipment {
                        id: lc.item_id,
                        level: lc.level,
                        promotion: lc.promotion,
                        rank: lc.rank,
                    }]
                })
                .unwrap_or_default();

            let relic_list = state
                .data
                .relics
                .iter()
                .filter(|r| r.equip_avatar == avatar_id)
                .map(|r| BattleRelic {
                    id: r.relic_id,
                    level: r.level,
                    main_affix_id: r.main_affix_id,
                    unique_id: r.internal_uid + 1,
                    sub_affix_list: r
                        .sub_affixes
                        .iter()
                        .map(|sa| RelicAffix {
                            affix_id: sa.sub_affix_id,
                            cnt: sa.count,
                            step: sa.step,
                        })
                        .collect(),
                    ..Default::default()
                })
                .collect();

            let mut avatar = BattleAvatar {
                avatar_type: AvatarType::AvatarFormalType as i32,
                id: a.avatar_id,
                level: a.level,
                rank: a.rank,
                index: idx as u32,
                hp: 10_000,
                promotion: a.promotion,
                world_level,
                enhanced_id: a.enhanced_id,
                sp_bar: Some(SpBarInfo { cur_sp, max_sp }),
                equipment_list,
                relic_list,
                ..Default::default()
            };
            avatar.skilltree_list = a
                .skills_by_anchor_type
                .iter()
                .map(|(point_id, level)| AvatarSkillTree {
                    point_id: *point_id,
                    level: *level,
                })
                .collect();
            Some(avatar)
        })
        .collect()
}

fn build_monster_waves(state: &GameServerState) -> Vec<SceneMonsterWave> {
    state
        .data
        .battle
        .monsters
        .iter()
        .enumerate()
        .map(|(idx, wave)| {
            let level = wave.iter().map(|m| m.level).max().unwrap_or(95);
            SceneMonsterWave {
                battle_stage_id: state.data.battle.stage_id,
                battle_wave_id: (idx as u32) + 1,
                monster_param: Some(SceneMonsterWaveParam {
                    level,
                    ..Default::default()
                }),
                monster_list: wave
                    .iter()
                    .flat_map(|m| {
                        std::iter::repeat_with(move || SceneMonster {
                            monster_id: m.monster_id,
                            max_hp: 0,
                            ..Default::default()
                        })
                        .take(m.amount as usize)
                    })
                    .collect(),
                ..Default::default()
            }
        })
        .collect()
}

fn build_stage_blessings(state: &GameServerState) -> Vec<BattleBuff> {
    state
        .data
        .battle
        .blessings
        .iter()
        .map(|b| make_skill_index_buff(b.id, b.level.max(1), u32::MAX))
        .collect()
}

fn add_global_passive_buffs(battle: &mut SceneBattleInfo, state: &GameServerState) {
    let lineup = {
        let guard = state.runtime.read().expect("runtime read");
        guard.lineup.clone()
    };
    if lineup.contains(&1407) {
        battle.buff_list.push(make_skill_index_buff(140703, 1, 1));
    }
    if lineup.contains(&1506) {
        battle.buff_list.push(make_skill_index_buff(150602, 1, 1));
    }
}

fn add_trigger_attack_buff(battle: &mut SceneBattleInfo, state: &GameServerState) {
    let leader_slot = {
        let guard = state.runtime.read().expect("runtime read");
        guard.leader_slot
    };
    // Keep this aligned with battle_mgr.zig idea: add an attacker trigger buff with SkillIndex=1.
    let mut buff = make_skill_index_buff(1000111, 1, leader_slot);
    buff.dynamic_values.insert("SkillIndex".to_string(), 1.0);
    battle.buff_list.push(buff);
}

fn add_battle_targets(battle: &mut SceneBattleInfo) {
    let stage_id = battle.stage_id;
    if is_pf_stage(stage_id) {
        battle.battle_target_info.insert(
            1,
            BattleTargetList {
                battle_target_list: vec![BattleTarget {
                    id: 10002,
                    progress: 0,
                    total_progress: 80000,
                }],
            },
        );
        for key in 2..=4 {
            battle.battle_target_info.insert(
                key,
                BattleTargetList {
                    battle_target_list: Vec::new(),
                },
            );
        }
        battle.battle_target_info.insert(
            5,
            BattleTargetList {
                battle_target_list: vec![
                    BattleTarget {
                        id: 2001,
                        progress: 0,
                        total_progress: 0,
                    },
                    BattleTarget {
                        id: 2002,
                        progress: 0,
                        total_progress: 0,
                    },
                ],
            },
        );
    } else if is_as_stage(stage_id) {
        battle.battle_target_info.insert(
            1,
            BattleTargetList {
                battle_target_list: vec![BattleTarget {
                    id: 90005,
                    progress: 2000,
                    total_progress: 0,
                }],
            },
        );
    }
}

fn add_su_resonance_event(battle: &mut SceneBattleInfo, state: &GameServerState) {
    if state.data.battle.battle_type.eq_ignore_ascii_case("SU")
        && state.data.battle.path_resonance_id != 0
    {
        battle.battle_event.push(BattleEventBattleInfo {
            battle_event_id: state.data.battle.path_resonance_id,
            status: Some(BattleEventProperty {
                sp_bar: Some(SpBarInfo {
                    cur_sp: 10_000,
                    max_sp: 10_000,
                }),
            }),
            ..Default::default()
        });
    }
}

fn collect_target_monster_ids(req: &SceneCastSkillCsReq) -> Vec<u32> {
    if !req.assist_monster_entity_id_list.is_empty() {
        return req.assist_monster_entity_id_list.clone();
    }
    if !req.hit_target_entity_id_list.is_empty() {
        return req.hit_target_entity_id_list.clone();
    }
    req.assist_monster_entity_info
        .iter()
        .flat_map(|v| v.entity_id_list.iter().copied())
        .collect()
}

fn get_battle_type(target_entity_id: u32, attacker_id: u32, skill_index: u32) -> MonsterBattleType {
    // Match battle.zig default behavior: mostly trigger battle.
    if skill_index != 1 {
        return MonsterBattleType::TriggerBattle;
    }
    if (1..=1000).contains(&attacker_id) {
        return MonsterBattleType::TriggerBattle;
    }
    if attacker_id >= 100_000 && target_entity_id <= 1_000_000 {
        return MonsterBattleType::DirectDieSkipBattle;
    }
    MonsterBattleType::TriggerBattle
}

fn is_pf_stage(stage_id: u32) -> bool {
    (30019000..=30019100).contains(&stage_id)
        || (30021000..=30021100).contains(&stage_id)
        || (30301000..=30399900).contains(&stage_id)
}

fn is_as_stage(stage_id: u32) -> bool {
    (420100..=420900).contains(&stage_id)
}

fn make_skill_index_buff(id: u32, level: u32, owner_index: u32) -> BattleBuff {
    let mut dynamic_values = HashMap::new();
    dynamic_values.insert("SkillIndex".to_string(), 0.0);
    BattleBuff {
        id,
        level,
        owner_index,
        wave_flag: u32::MAX,
        target_index_list: vec![0],
        dynamic_values,
    }
}

fn allocate_battle_id(state: &GameServerState, world_level: u32) -> u32 {
    let mut guard = state.runtime.write().expect("runtime write");
    let id = guard.next_battle_id;
    guard.next_battle_id = guard.next_battle_id.saturating_add(1).max(1);
    guard.last_world_level = world_level;
    id
}

fn now_seed() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos()
}
