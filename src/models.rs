use serde::Serialize;
use std::sync::Arc;
use crate::db::Db;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
}

/// Represents the mood/happiness level of the Tamagotchi (1-10).
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
    pub fn as_text(&self, lang: &str) -> &'static str {
        match lang {
            "en" => match self {
                Mood::Abattu => "dejected",
                Mood::Chagrine => "sorrowful",
                Mood::Deprime => "depressed",
                Mood::Triste => "sad",
                Mood::Neutre => "neutral",
                Mood::Content => "content",
                Mood::Heureux => "happy",
                Mood::EnPleineForme => "in great shape",
                Mood::Euphorique => "euphoric",
                Mood::Radieux => "radiant",
            },
            "de" => match self {
                Mood::Abattu => "niedergeschlagen",
                Mood::Chagrine => "bekümmert",
                Mood::Deprime => "deprimiert",
                Mood::Triste => "traurig",
                Mood::Neutre => "neutral",
                Mood::Content => "zufrieden",
                Mood::Heureux => "glücklich",
                Mood::EnPleineForme => "in Bestform",
                Mood::Euphorique => "euphorisch",
                Mood::Radieux => "strahlend",
            },
            _ => match self {
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
    }

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
                if level < 1 { Mood::Abattu } else { Mood::Radieux }
            }
        }
    }
}

/// Represents the playfulness level of the Tamagotchi (1-10).
/// Increases by 1 every 3 total play interactions.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum Playfulness {
    Ennuye = 1,
    Indifferent = 2,
    Distrait = 3,
    Intrigue = 4,
    Amuse = 5,
    Enthousiaste = 6,
    Passionne = 7,
    Exalte = 8,
    Hilare = 9,
    Extatique = 10,
}

impl Playfulness {
    pub fn as_text(&self, lang: &str) -> &'static str {
        match lang {
            "en" => match self {
                Playfulness::Ennuye => "bored",
                Playfulness::Indifferent => "indifferent",
                Playfulness::Distrait => "distracted",
                Playfulness::Intrigue => "intrigued",
                Playfulness::Amuse => "amused",
                Playfulness::Enthousiaste => "enthusiastic",
                Playfulness::Passionne => "passionate",
                Playfulness::Exalte => "elated",
                Playfulness::Hilare => "hilarious",
                Playfulness::Extatique => "ecstatic",
            },
            "de" => match self {
                Playfulness::Ennuye => "gelangweilt",
                Playfulness::Indifferent => "gleichgültig",
                Playfulness::Distrait => "abgelenkt",
                Playfulness::Intrigue => "fasziniert",
                Playfulness::Amuse => "amüsiert",
                Playfulness::Enthousiaste => "enthusiastisch",
                Playfulness::Passionne => "leidenschaftlich",
                Playfulness::Exalte => "begeistert",
                Playfulness::Hilare => "urkomisch",
                Playfulness::Extatique => "ekstatisch",
            },
            _ => match self {
                Playfulness::Ennuye => "ennuyé",
                Playfulness::Indifferent => "indifférent",
                Playfulness::Distrait => "distrait",
                Playfulness::Intrigue => "intrigué",
                Playfulness::Amuse => "amusé",
                Playfulness::Enthousiaste => "enthousiaste",
                Playfulness::Passionne => "passionné",
                Playfulness::Exalte => "exalté",
                Playfulness::Hilare => "hilare",
                Playfulness::Extatique => "extatique",
            }
        }
    }

    pub fn from_level(level: i32) -> Self {
        match level {
            1 => Playfulness::Ennuye,
            2 => Playfulness::Indifferent,
            3 => Playfulness::Distrait,
            4 => Playfulness::Intrigue,
            5 => Playfulness::Amuse,
            6 => Playfulness::Enthousiaste,
            7 => Playfulness::Passionne,
            8 => Playfulness::Exalte,
            9 => Playfulness::Hilare,
            10 => Playfulness::Extatique,
            _ => {
                if level < 1 { Playfulness::Ennuye } else { Playfulness::Extatique }
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
    pub can_play: bool,
    pub player_plays_today: i32,
    pub plays_today: i32,
    pub playfulness_text: String,
}

/// API response for a feed action.
#[derive(Serialize)]
pub struct FeedResponse {
    pub message: String,
    pub level_id: i32,
    pub mood_text: String,
    pub feeds_today: i32,
}

/// API response for a play action.
#[derive(Serialize)]
pub struct PlayResponse {
    pub message: String,
    pub playfulness_text: String,
    pub plays_today: i32,
    pub player_plays_today: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mood_as_text() {
        assert_eq!(Mood::Abattu.as_text("fr"), "abattu");
        assert_eq!(Mood::Neutre.as_text("en"), "neutral");
        assert_eq!(Mood::Radieux.as_text("de"), "strahlend");
    }

    #[test]
    fn test_mood_from_level() {
        assert_eq!(Mood::from_level(1), Mood::Abattu);
        assert_eq!(Mood::from_level(5), Mood::Neutre);
        assert_eq!(Mood::from_level(10), Mood::Radieux);
        assert_eq!(Mood::from_level(-5), Mood::Abattu);
        assert_eq!(Mood::from_level(15), Mood::Radieux);
    }

    #[test]
    fn test_playfulness_as_text() {
        assert_eq!(Playfulness::Ennuye.as_text("fr"), "ennuyé");
        assert_eq!(Playfulness::Amuse.as_text("en"), "amused");
        assert_eq!(Playfulness::Extatique.as_text("de"), "ekstatisch");
    }

    #[test]
    fn test_playfulness_from_level() {
        assert_eq!(Playfulness::from_level(1), Playfulness::Ennuye);
        assert_eq!(Playfulness::from_level(5), Playfulness::Amuse);
        assert_eq!(Playfulness::from_level(10), Playfulness::Extatique);
        assert_eq!(Playfulness::from_level(0), Playfulness::Ennuye);
        assert_eq!(Playfulness::from_level(99), Playfulness::Extatique);
    }
}
