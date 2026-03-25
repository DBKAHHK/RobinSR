use std::{collections::HashMap, fs, io};

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct GameData {
    pub uid: u32,
    pub token: String,
    pub nickname: String,
    pub avatars: Vec<AvatarRecord>,
    pub lightcones: Vec<LightconeRecord>,
    pub relics: Vec<RelicRecord>,
    pub battle: BattleConfigRecord,
    pub mc_id: u32,
    pub march_id: u32,
    pub lineup: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct AvatarRecord {
    pub avatar_id: u32,
    pub level: u32,
    pub promotion: u32,
    pub rank: u32,
    pub enhanced_id: u32,
    pub sp_value: u32,
    pub sp_max: u32,
    pub skills_by_anchor_type: Vec<(u32, u32)>,
}

#[derive(Debug, Clone)]
pub struct LightconeRecord {
    pub internal_uid: u32,
    pub equip_avatar: u32,
    pub item_id: u32,
    pub promotion: u32,
    pub rank: u32,
    pub level: u32,
}

#[derive(Debug, Clone)]
pub struct RelicRecord {
    pub internal_uid: u32,
    pub equip_avatar: u32,
    pub level: u32,
    pub main_affix_id: u32,
    pub relic_id: u32,
    pub sub_affixes: Vec<RelicSubAffixRecord>,
}

#[derive(Debug, Clone)]
pub struct RelicSubAffixRecord {
    pub sub_affix_id: u32,
    pub count: u32,
    pub step: u32,
}

#[derive(Debug, Clone)]
pub struct BattleConfigRecord {
    pub battle_type: String,
    pub stage_id: u32,
    pub cycle_count: u32,
    pub path_resonance_id: u32,
    pub monsters: Vec<Vec<BattleMonsterRecord>>,
    pub blessings: Vec<BattleBlessingRecord>,
}

#[derive(Debug, Clone)]
pub struct BattleMonsterRecord {
    pub monster_id: u32,
    pub amount: u32,
    pub level: u32,
}

#[derive(Debug, Clone)]
pub struct BattleBlessingRecord {
    pub id: u32,
    pub level: u32,
}

#[derive(Debug, Deserialize)]
struct FreeSrData {
    key: Option<String>,
    avatars: HashMap<String, AvatarEntry>,
    #[serde(default)]
    lightcones: Vec<LightconeEntry>,
    #[serde(default)]
    relics: Vec<RelicEntry>,
    #[serde(default)]
    battle_config: BattleConfigEntry,
}

#[derive(Debug, Deserialize)]
struct AvatarEntry {
    avatar_id: u32,
    level: u32,
    promotion: u32,
    enhanced_id: Option<u32>,
    sp_value: Option<u32>,
    sp_max: Option<u32>,
    data: Option<AvatarInnerData>,
}

#[derive(Debug, Deserialize)]
struct AvatarInnerData {
    rank: Option<u32>,
    #[serde(default)]
    skills_by_anchor_type: HashMap<String, u32>,
}

#[derive(Debug, Deserialize)]
struct LightconeEntry {
    internal_uid: u32,
    equip_avatar: u32,
    item_id: u32,
    promotion: u32,
    rank: u32,
    level: u32,
}

#[derive(Debug, Deserialize)]
struct RelicEntry {
    internal_uid: u32,
    equip_avatar: u32,
    level: u32,
    main_affix_id: u32,
    relic_id: u32,
    #[serde(default)]
    sub_affixes: Vec<RelicSubAffixEntry>,
}

#[derive(Debug, Deserialize)]
struct RelicSubAffixEntry {
    sub_affix_id: u32,
    count: u32,
    step: u32,
}

#[derive(Debug, Deserialize, Default)]
struct BattleConfigEntry {
    #[serde(default)]
    battle_type: String,
    #[serde(default)]
    cycle_count: u32,
    #[serde(default)]
    stage_id: u32,
    #[serde(default)]
    path_resonance_id: u32,
    #[serde(default)]
    monsters: Vec<Vec<BattleMonsterEntry>>,
    #[serde(default)]
    blessings: Vec<BattleBlessingEntry>,
}

#[derive(Debug, Deserialize)]
struct BattleMonsterEntry {
    monster_id: u32,
    #[serde(default = "default_monster_amount")]
    amount: u32,
    #[serde(default)]
    level: u32,
}

#[derive(Debug, Deserialize)]
struct BattleBlessingEntry {
    id: u32,
    #[serde(default = "default_blessing_level")]
    level: u32,
}

fn default_blessing_level() -> u32 {
    1
}

fn default_monster_amount() -> u32 {
    1
}

#[derive(Debug, Deserialize)]
struct PersistentFile {
    avatar: PersistentAvatar,
}

#[derive(Debug, Deserialize)]
struct PersistentAvatar {
    mc_id: String,
    march_id: String,
    lineup: Vec<u32>,
}

pub fn load_data(freesr_path: &str, persistent_path: &str) -> io::Result<GameData> {
    let freesr_raw = fs::read_to_string(freesr_path)?;
    let mut freesr: FreeSrData = serde_json::from_str(&freesr_raw)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("freesr-data.json: {e}")))?;

    let persistent_raw = fs::read_to_string(persistent_path)?;
    let persistent: PersistentFile = serde_json::from_str(&persistent_raw)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("persistent.json: {e}")))?;

    let mut avatars = Vec::with_capacity(freesr.avatars.len());
    for (_, v) in freesr.avatars.drain() {
        let inner = v.data;
        let mut skills: Vec<(u32, u32)> = inner
            .as_ref()
            .map(|d| {
                d.skills_by_anchor_type
                    .iter()
                    .filter_map(|(k, lv)| k.parse::<u32>().ok().map(|id| (id, *lv)))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        skills.sort_by_key(|(id, _)| *id);

        avatars.push(AvatarRecord {
            avatar_id: v.avatar_id,
            level: v.level,
            promotion: v.promotion,
            rank: inner.and_then(|d| d.rank).unwrap_or(1),
            enhanced_id: v.enhanced_id.unwrap_or(0),
            sp_value: v.sp_value.unwrap_or(0),
            sp_max: v.sp_max.unwrap_or(10_000),
            skills_by_anchor_type: skills,
        });
    }
    avatars.sort_by_key(|a| a.avatar_id);

    let lightcones = freesr
        .lightcones
        .into_iter()
        .map(|v| LightconeRecord {
            internal_uid: v.internal_uid,
            equip_avatar: v.equip_avatar,
            item_id: v.item_id,
            promotion: v.promotion,
            rank: v.rank,
            level: v.level,
        })
        .collect();

    let relics = freesr
        .relics
        .into_iter()
        .map(|v| RelicRecord {
            internal_uid: v.internal_uid,
            equip_avatar: v.equip_avatar,
            level: v.level,
            main_affix_id: v.main_affix_id,
            relic_id: v.relic_id,
            sub_affixes: v
                .sub_affixes
                .into_iter()
                .map(|s| RelicSubAffixRecord {
                    sub_affix_id: s.sub_affix_id,
                    count: s.count,
                    step: s.step,
                })
                .collect(),
        })
        .collect();

    let battle = BattleConfigRecord {
        battle_type: freesr.battle_config.battle_type,
        stage_id: freesr.battle_config.stage_id,
        cycle_count: freesr.battle_config.cycle_count,
        path_resonance_id: freesr.battle_config.path_resonance_id,
        monsters: freesr
            .battle_config
            .monsters
            .into_iter()
            .map(|wave| {
                wave.into_iter()
                    .map(|m| BattleMonsterRecord {
                        monster_id: m.monster_id,
                        amount: m.amount.max(1),
                        level: m.level,
                    })
                    .collect()
            })
            .collect(),
        blessings: freesr
            .battle_config
            .blessings
            .into_iter()
            .map(|b| BattleBlessingRecord {
                id: b.id,
                level: b.level,
            })
            .collect(),
    };

    let mc_id = persistent.avatar.mc_id.parse::<u32>().map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, format!("persistent.avatar.mc_id: {e}"))
    })?;
    let march_id = persistent.avatar.march_id.parse::<u32>().map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, format!("persistent.avatar.march_id: {e}"))
    })?;

    Ok(GameData {
        uid: 333,
        token: freesr.key.unwrap_or_else(|| "arayashiki".to_string()),
        nickname: "RobinSR".to_string(),
        avatars,
        lightcones,
        relics,
        battle,
        mc_id,
        march_id,
        lineup: persistent.avatar.lineup,
    })
}
