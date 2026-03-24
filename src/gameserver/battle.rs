use crate::proto::{BattleEndStatus, GetCurBattleInfoScRsp};

pub fn on_get_cur_battle_info() -> GetCurBattleInfoScRsp {
    GetCurBattleInfoScRsp {
        last_end_status: BattleEndStatus::BattleEndQuit as i32,
        retcode: 0,
        ..Default::default()
    }
}
