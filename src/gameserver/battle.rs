use prost::Message;

use crate::proto::{
    BattleEndStatus, GetCurBattleInfoScRsp, PveBattleResultCsReq, PveBattleResultScRsp,
    StartCocoonStageCsReq, StartCocoonStageScRsp,
};

pub fn on_get_cur_battle_info() -> GetCurBattleInfoScRsp {
    GetCurBattleInfoScRsp {
        last_end_status: BattleEndStatus::BattleEndQuit as i32,
        retcode: 0,
        ..Default::default()
    }
}

pub fn on_start_cocoon_stage(body: &[u8]) -> StartCocoonStageScRsp {
    let req = StartCocoonStageCsReq::decode(body).unwrap_or_default();
    StartCocoonStageScRsp {
        wave: req.wave,
        prop_entity_id: req.prop_entity_id,
        cocoon_id: req.cocoon_id,
        challenge_cnt: 1,
        retcode: 0,
        ..Default::default()
    }
}

pub fn on_pve_battle_result(body: &[u8]) -> PveBattleResultScRsp {
    let req = PveBattleResultCsReq::decode(body).unwrap_or_default();
    PveBattleResultScRsp {
        battle_id: req.battle_id,
        stage_id: req.stage_id,
        end_status: req.end_status,
        retcode: 0,
        ..Default::default()
    }
}
