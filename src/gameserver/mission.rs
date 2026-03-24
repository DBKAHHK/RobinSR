use prost::Message;

use crate::proto::{GetMissionStatusCsReq, GetMissionStatusScRsp, Mission, MissionStatus};

pub fn on_get_mission_status(body: &[u8]) -> GetMissionStatusScRsp {
    let req = GetMissionStatusCsReq::decode(body).unwrap_or(GetMissionStatusCsReq {
        main_mission_id_list: Vec::new(),
        sub_mission_id_list: Vec::new(),
    });

    GetMissionStatusScRsp {
        finished_main_mission_id_list: req.main_mission_id_list,
        sub_mission_status_list: req
            .sub_mission_id_list
            .into_iter()
            .map(|id| Mission {
                id,
                progress: 1,
                status: MissionStatus::MissionFinish as i32,
            })
            .collect(),
        retcode: 0,
        ..Default::default()
    }
}
