use serde::{Deserialize, Serialize};

pub trait MunModel {
    fn from_str(data: &str) -> Self
    where
        Self: Sized;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Direction {
    #[serde(rename = "N")]
    N,
    #[serde(rename = "E")]
    E,
    #[serde(rename = "S")]
    S,
    #[serde(rename = "W")]
    W,
}

impl Direction {
    pub fn to_movement_json(&self) -> String {
        format!(
            "{{ direction: \"{}\" }}",
            match self {
                Direction::N => "N",
                Direction::E => "E",
                Direction::S => "S",
                Direction::W => "W",
            }
        )
    }

    pub fn to_string(&self) -> String {
        match self {
            Direction::N => String::from("N"),
            Direction::E => String::from("E"),
            Direction::S => String::from("S"),
            Direction::W => String::from("W"),
        }
    }
}

impl MunModel for Direction {
    fn from_str(data: &str) -> Direction {
        serde_json::from_str(data).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EntityType {
    #[serde(rename = "MONSTRE")]
    Monster,
    #[serde(rename = "JOUEUR")]
    Player,
}

impl MunModel for EntityType {
    fn from_str(data: &str) -> EntityType {
        serde_json::from_str(data).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ErrorType {
    #[serde(rename = "MORT")]
    Dead,
    #[serde(rename = "MUR")]
    Wall,
    #[serde(rename = "DIFFSALLE")]
    DiffRoom,
}

impl MunModel for ErrorType {
    fn from_str(data: &str) -> ErrorType {
        serde_json::from_str(data).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Room {
    pub description: String,
    #[serde(rename = "passages")]
    pub paths: Vec<Direction>,
    #[serde(rename = "entites")]
    pub entities: Vec<String>,
}

impl MunModel for Room {
    fn from_str(data: &str) -> Room {
        serde_json::from_str(data).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Status {
    pub guid: String,
    #[serde(rename = "totalvie")]
    pub total_life: u32,
    #[serde(rename = "salle")]
    pub room: Room,
}

impl MunModel for Status {
    fn from_str(data: &str) -> Status {
        serde_json::from_str(data).unwrap()
    }
}

impl Status {
    pub fn get_paths_string(&self) -> String {
        let paths = &self.room.paths;

        match paths.len() {
            0 => String::from("None"),
            _ => {
                let mut s = String::new();
                s += &paths[0].to_string();

                for path in paths.iter().skip(1) {
                    s += &format!(", {}", path.to_string());
                }

                s
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entity {
    pub description: String,
    pub r#type: EntityType,
    #[serde(rename = "vie")]
    pub life: u32,
    #[serde(rename = "totalvie")]
    pub total_life: u32,
}

impl MunModel for Entity {
    fn from_str(data: &str) -> Entity {
        serde_json::from_str(data).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Fighter {
    pub guid: String,
    #[serde(rename = "degats")]
    pub damage: u32,
    #[serde(rename = "vie")]
    pub life: u32,
}

impl MunModel for Fighter {
    fn from_str(data: &str) -> Fighter {
        serde_json::from_str(data).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Fight {
    #[serde(rename = "attaquant")]
    pub attacker: Fighter,
    #[serde(rename = "attaque")]
    pub defender: Fighter,
}

impl MunModel for Fight {
    fn from_str(data: &str) -> Fight {
        serde_json::from_str(data).unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ErrorDetail {
    pub r#type: Option<ErrorType>,
    pub message: String,
}

impl MunModel for ErrorDetail {
    fn from_str(data: &str) -> ErrorDetail {
        serde_json::from_str(data).unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct Error {
    pub code: Option<u16>,
    pub detail: ErrorDetail,
}
