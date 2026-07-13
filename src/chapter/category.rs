use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    StoriesOfLegend,
    RegularEventStages,
    CollabStages,
    EmpireOfCats,
    IntoTheFuture,
    CatsOfTheCosmos,
    EventStages,
    ContinuationStages,
    DojoHallOfInitiates,
    TowersAndCitadels,
    DojoRankingEvents,
    ChallengeBattle,
    UncannyLegends,
    CataminStages,
    LegendQuest,
    ZombieOutbreaks,
    GauntletStages,
    EnigmaStages,
    CollabGauntletStages,
    AkuRealms,
    BehemothCulling,
    Labyrinth,
    ZeroLegends,
    OtherworldColosseum,
    CatclawChampionships,
    Unknown(String),
}

impl Default for Category {
    fn default() -> Self {
        Self::Unknown(String::new())
    }
}

impl Category {
    pub fn map_prefix(&self) -> String {
        match self {
            Self::StoriesOfLegend      => "N".to_string(),
            Self::RegularEventStages   => "S".to_string(),
            Self::CollabStages         => "C".to_string(),
            Self::EmpireOfCats         => "EC".to_string(),
            Self::IntoTheFuture        => "W".to_string(),
            Self::CatsOfTheCosmos      => "Space".to_string(),
            Self::EventStages          => "E".to_string(),
            Self::ContinuationStages   => "EX".to_string(),
            Self::DojoHallOfInitiates  => "T".to_string(),
            Self::TowersAndCitadels    => "V".to_string(),
            Self::DojoRankingEvents    => "R".to_string(),
            Self::ChallengeBattle      => "M".to_string(),
            Self::UncannyLegends       => "NA".to_string(),
            Self::CataminStages        => "B".to_string(),
            Self::LegendQuest          => "D".to_string(),
            Self::ZombieOutbreaks      => "Z".to_string(),
            Self::GauntletStages       => "A".to_string(),
            Self::EnigmaStages         => "H".to_string(),
            Self::CollabGauntletStages => "CA".to_string(),
            Self::AkuRealms            => "DM".to_string(),
            Self::BehemothCulling      => "Q".to_string(),
            Self::Labyrinth            => "L".to_string(),
            Self::ZeroLegends          => "ND".to_string(),
            Self::OtherworldColosseum  => "SR".to_string(),
            Self::CatclawChampionships => "G".to_string(),
            Self::Unknown(prefix)      => prefix.clone(),
        }
    }

    pub fn image_prefix(&self) -> String {
        self.map_prefix().to_lowercase()
    }

    pub fn stage_prefix(&self) -> Vec<String> {
        let base = self.map_prefix();
        let mut prefixes = vec![base.clone()];

        match self {
            Self::StoriesOfLegend      => prefixes.push("RN".to_string()),
            Self::RegularEventStages   => prefixes.push("RS".to_string()),
            Self::CollabStages         => prefixes.push("RC".to_string()),
            Self::EmpireOfCats         => prefixes.push("".to_string()),
            Self::EventStages          => prefixes.push("RE".to_string()),
            Self::DojoHallOfInitiates  => prefixes.push("RT".to_string()),
            Self::TowersAndCitadels    => prefixes.push("RV".to_string()),
            Self::DojoRankingEvents    => prefixes.push("RR".to_string()),
            Self::ChallengeBattle      => prefixes.push("RM".to_string()),
            Self::UncannyLegends       => prefixes.push("RNA".to_string()),
            Self::CataminStages        => prefixes.push("RB".to_string()),
            Self::GauntletStages       => prefixes.push("RA".to_string()),
            Self::EnigmaStages         => prefixes.push("RH".to_string()),
            Self::CollabGauntletStages => prefixes.push("RCA".to_string()),
            Self::AkuRealms            => prefixes.push("DM".to_string()),
            Self::BehemothCulling      => prefixes.push("RQ".to_string()),
            Self::ZeroLegends          => prefixes.push("RND".to_string()),
            Self::OtherworldColosseum  => prefixes.push("RSR".to_string()),
            Self::Unknown(prefix)      => {
                let upper = prefix.to_uppercase();
                if upper.starts_with('R') && upper.len() > 1 {
                    prefixes.push(upper[1..].to_string());
                }
            },
            _ => {}
        }

        prefixes
    }

    pub fn from_prefix(prefix: &str) -> Self {
        match prefix.to_uppercase().as_str() {
            "N"     | "RN"  => Self::StoriesOfLegend,
            "S"     | "RS"  => Self::RegularEventStages,
            "C"     | "RC"  => Self::CollabStages,
            "EC"    | ""    => Self::EmpireOfCats,
            "W"             => Self::IntoTheFuture,
            "SPACE"         => Self::CatsOfTheCosmos,
            "E"     | "RE"  => Self::EventStages,
            "EX"            => Self::ContinuationStages,
            "T"     | "RT"  => Self::DojoHallOfInitiates,
            "V"     | "RV"  => Self::TowersAndCitadels,
            "R"     | "RR"  => Self::DojoRankingEvents,
            "M"     | "RM"  => Self::ChallengeBattle,
            "NA"    | "RNA" => Self::UncannyLegends,
            "B"     | "RB"  => Self::CataminStages,
            "D"             => Self::LegendQuest,
            "Z"             => Self::ZombieOutbreaks,
            "A"     | "RA"  => Self::GauntletStages,
            "H"     | "RH"  => Self::EnigmaStages,
            "CA"    | "RCA" => Self::CollabGauntletStages,
            "DM"            => Self::AkuRealms,
            "Q"     | "RQ"  => Self::BehemothCulling,
            "L"             => Self::Labyrinth,
            "ND"    | "RND" => Self::ZeroLegends,
            "SR"    | "RSR" => Self::OtherworldColosseum,
            "G"             => Self::CatclawChampionships,
            _               => Self::Unknown(prefix.to_string()),
        }
    }

    pub fn base_id(&self) -> Option<u32> {
        match self {
            Self::StoriesOfLegend      => Some(0),
            Self::RegularEventStages   => Some(1),
            Self::CollabStages         => Some(2),
            Self::EmpireOfCats         => None,
            Self::IntoTheFuture        => None,
            Self::CatsOfTheCosmos      => None,
            Self::EventStages          => Some(4),
            Self::ContinuationStages   => Some(4),
            Self::DojoHallOfInitiates  => Some(6),
            Self::TowersAndCitadels    => Some(7),
            Self::DojoRankingEvents    => Some(11),
            Self::ChallengeBattle      => Some(12),
            Self::UncannyLegends       => Some(13),
            Self::CataminStages        => Some(14),
            Self::LegendQuest          => Some(16),
            Self::ZombieOutbreaks      => None,
            Self::GauntletStages       => Some(24),
            Self::EnigmaStages         => Some(25),
            Self::CollabGauntletStages => Some(27),
            Self::AkuRealms            => Some(30),
            Self::BehemothCulling      => Some(31),
            Self::Labyrinth            => Some(33),
            Self::ZeroLegends          => Some(34),
            Self::OtherworldColosseum  => Some(36),
            Self::CatclawChampionships => Some(37),
            Self::Unknown(_)           => None,
        }
    }

    pub fn global_map_id(&self, local_map_id: u32) -> Option<u32> {
        self.base_id().map(|base| (base * 1000) + local_map_id)
    }
}