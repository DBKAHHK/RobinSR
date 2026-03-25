use crate::proto::{Equipment, GetBagScRsp, Material, Relic, RelicAffix};

use super::GameServerState;

pub fn on_get_bag(state: &GameServerState) -> GetBagScRsp {
    GetBagScRsp {
        equipment_list: state
            .data
            .lightcones
            .iter()
            .map(|lc| Equipment {
                dress_avatar_id: lc.equip_avatar,
                exp: 0,
                is_protected: false,
                level: lc.level,
                promotion: lc.promotion,
                rank: lc.rank,
                tid: lc.item_id,
                unique_id: lc.internal_uid + 2000,
            })
            .collect(),
        relic_list: state
            .data
            .relics
            .iter()
            .map(|r| Relic {
                dress_avatar_id: r.equip_avatar,
                exp: 0,
                is_protected: false,
                level: r.level,
                main_affix_id: r.main_affix_id,
                tid: r.relic_id,
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
            .collect(),
        material_list: vec![
            Material {
                tid: 101,
                num: 999999,
                expire_time: 0,
            },
            Material {
                tid: 102,
                num: 999999,
                expire_time: 0,
            },
        ],
        retcode: 0,
        ..Default::default()
    }
}
