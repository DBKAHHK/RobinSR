use prost::Message;

use crate::proto::{
    DisplayAvatarVec, GetBasicInfoScRsp, GetPlayerBoardDataScRsp, HeadFrameInfo, HeadIconData,
    PlayerBasicInfo, PlayerGetTokenScRsp, PlayerHeartBeatCsReq, PlayerHeartBeatScRsp,
    PlayerLoginScRsp, PlayerSettingInfo, SetClientPausedCsReq, SetClientPausedScRsp,
};

use super::{now_ms, GameServerState};

pub fn on_player_get_token(state: &GameServerState) -> PlayerGetTokenScRsp {
    PlayerGetTokenScRsp {
        retcode: 0,
        uid: state.data.uid,
        ..Default::default()
    }
}

pub fn on_player_heart_beat(body: &[u8]) -> PlayerHeartBeatScRsp {
    let client_time_ms = PlayerHeartBeatCsReq::decode(body)
        .map(|v| v.client_time_ms)
        .unwrap_or(0);

    PlayerHeartBeatScRsp {
        retcode: 0,
        client_time_ms,
        server_time_ms: now_ms(),
        ..Default::default()
    }
}

pub fn on_player_login(state: &GameServerState) -> PlayerLoginScRsp {
    PlayerLoginScRsp {
        basic_info: Some(PlayerBasicInfo {
            nickname: state.data.nickname.clone(),
            level: 70,
            stamina: 240,
            world_level: 6,
            ..Default::default()
        }),
        server_timestamp_ms: now_ms(),
        stamina: 240,
        retcode: 0,
        ..Default::default()
    }
}

pub fn on_get_basic_info() -> GetBasicInfoScRsp {
    GetBasicInfoScRsp {
        cur_day: 1,
        player_setting_info: Some(PlayerSettingInfo::default()),
        is_gender_set: true,
        gender: 1,
        retcode: 0,
        ..Default::default()
    }
}

pub fn on_get_player_board_data(state: &GameServerState) -> GetPlayerBoardDataScRsp {
    let (head_icon, march_id) = {
        let guard = state.runtime.read().expect("runtime read");
        (guard.mc_id, guard.march_id)
    };
    GetPlayerBoardDataScRsp {
        signature: "RobinSR".to_string(),
        current_head_icon_id: head_icon,
        unlocked_head_icon_list: vec![HeadIconData { id: head_icon }, HeadIconData { id: march_id }],
        head_frame_info: Some(HeadFrameInfo {
            head_frame_item_id: 226004,
            head_frame_expire_time: (now_ms() + 86_400_000) as i64,
        }),
        current_personal_card_id: 253001,
        unlocked_personal_card_list: vec![253001],
        display_avatar_vec: Some(DisplayAvatarVec {
            is_display: false,
            ..Default::default()
        }),
        retcode: 0,
        ..Default::default()
    }
}

pub fn on_set_client_paused(state: &GameServerState, body: &[u8]) -> SetClientPausedScRsp {
    let paused = SetClientPausedCsReq::decode(body)
        .map(|v| v.paused)
        .unwrap_or(false);

    if let Ok(mut guard) = state.runtime.write() {
        guard.client_paused = paused;
    }

    SetClientPausedScRsp {
        paused,
        retcode: 0,
        ..Default::default()
    }
}
