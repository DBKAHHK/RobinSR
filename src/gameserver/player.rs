use prost::Message;

use crate::proto::{
    ContentPackageData, ContentPackageGetDataScRsp, ContentPackageInfo, ContentPackageStatus,
    DisplayAvatarVec, GetBasicInfoScRsp, GetPlayerBoardDataScRsp, HeadFrameInfo, HeadIconData,
    PlayerBasicInfo, PlayerGetTokenScRsp, PlayerHeartBeatCsReq, PlayerHeartBeatScRsp, PlayerLoginCsReq,
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

const CONTENT_PACKAGE_IDS: &[u32] = &[
    200001, 200002, 200003, 200004, 200005, 200006, 200007, 150017, 150015, 150021, 150018,
    130011, 130012, 130013, 150025, 140006, 150026, 130014, 150034, 150029, 150035, 150041,
    150039, 150045, 150057, 150042, 150067, 150064, 150063,
];

pub fn on_content_package_get_data() -> ContentPackageGetDataScRsp {
    ContentPackageGetDataScRsp {
        retcode: 0,
        data: Some(ContentPackageData {
            cur_content_id: 0,
            content_package_list: CONTENT_PACKAGE_IDS
                .iter()
                .map(|id| ContentPackageInfo {
                    content_id: *id,
                    status: ContentPackageStatus::Finished as i32,
                })
                .collect(),
            ..Default::default()
        }),
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

pub fn on_player_login(state: &GameServerState, body: &[u8]) -> PlayerLoginScRsp {
    let login_random = PlayerLoginCsReq::decode(body)
        .map(|v| v.login_random)
        .unwrap_or(0);

    PlayerLoginScRsp {
        basic_info: Some(PlayerBasicInfo {
            nickname: state.data.nickname.clone(),
            level: 70,
            stamina: 240,
            world_level: 6,
            ..Default::default()
        }),
        login_random,
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
