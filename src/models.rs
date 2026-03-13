use serde::Serialize;
use std::sync::Arc;
use crate::db::Db;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
}

/// Represents the mood/level of the Tamagotchi.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum Mood {
    Abattu = 1,
    Chagrine = 2,
    Deprime = 3,
    Triste = 4,
    Neutre = 5,
    Content = 6,
    Heureux = 7,
    EnPleineForme = 8,
    Euphorique = 9,
    Radieux = 10,
}

impl Mood {
    /// Returns the French text representation of the mood.
    pub fn as_text(&self) -> &'static str {
        match self {
            Mood::Abattu => "abattu",
            Mood::Chagrine => "chagriné",
            Mood::Deprime => "déprimé",
            Mood::Triste => "triste",
            Mood::Neutre => "neutre",
            Mood::Content => "content",
            Mood::Heureux => "heureux",
            Mood::EnPleineForme => "en pleine forme",
            Mood::Euphorique => "euphorique",
            Mood::Radieux => "radieux",
        }
    }

    /// Converts a level (1-10) to a Mood.
    pub fn from_level(level: i32) -> Self {
        match level {
            1 => Mood::Abattu,
            2 => Mood::Chagrine,
            3 => Mood::Deprime,
            4 => Mood::Triste,
            5 => Mood::Neutre,
            6 => Mood::Content,
            7 => Mood::Heureux,
            8 => Mood::EnPleineForme,
            9 => Mood::Euphorique,
            10 => Mood::Radieux,
            _ => {
                if level < 1 {
                    Mood::Abattu
                } else {
                    Mood::Radieux
                }
            }
        }
    }
}

/// API response for the current status.
#[derive(Serialize)]
pub struct StatusResponse {
    pub level_id: i32,
    pub mood_text: String,
    pub has_fed_today: bool,
    pub feeds_today: i32,
}

/// API response for a feed action.
#[derive(Serialize)]
pub struct FeedResponse {
    pub message: String,
    pub level_id: i32,
    pub mood_text: String,
    pub feeds_today: i32,
}
