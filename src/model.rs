use serde::{Deserialize, Serialize};

static ERROR_DESERIALIZATION: &'static str = "Error while deserializing object";

pub trait MunModel {
    fn from_str(data: &str) -> Result<Self, Error>
    where
        Self: Sized;
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
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
            "{{ \"direction\": \"{}\" }}",
            match self {
                Direction::N => "N",
                Direction::E => "E",
                Direction::S => "S",
                Direction::W => "W",
            }
        )
    }
}

impl MunModel for Direction {
    fn from_str(data: &str) -> Result<Direction, Error> {
        match serde_json::from_str(data) {
            Ok(object) => Ok(object),
            Err(_) => Err(Error::from_error_string(ERROR_DESERIALIZATION.to_string())),
        }
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
    fn from_str(data: &str) -> Result<EntityType, Error> {
        match serde_json::from_str(data) {
            Ok(object) => Ok(object),
            Err(_) => Err(Error::from_error_string(ERROR_DESERIALIZATION.to_string())),
        }
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
    fn from_str(data: &str) -> Result<ErrorType, Error> {
        match serde_json::from_str(data) {
            Ok(object) => Ok(object),
            Err(_) => Err(Error::from_error_string(ERROR_DESERIALIZATION.to_string())),
        }
    }
}

impl ErrorType {
    pub fn to_string(&self) -> String {
        match self {
            ErrorType::Dead => String::from("Death"),
            ErrorType::Wall => String::from("Wall"),
            ErrorType::DiffRoom => String::from("Different room"),
        }
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
    fn from_str(data: &str) -> Result<Room, Error> {
        match serde_json::from_str(data) {
            Ok(object) => Ok(object),
            Err(_) => Err(Error::from_error_string(ERROR_DESERIALIZATION.to_string())),
        }
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
    fn from_str(data: &str) -> Result<Status, Error> {
        match serde_json::from_str(data) {
            Ok(object) => Ok(object),
            Err(_) => Err(Error::from_error_string(ERROR_DESERIALIZATION.to_string())),
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
    fn from_str(data: &str) -> Result<Entity, Error> {
        match serde_json::from_str(data) {
            Ok(object) => Ok(object),
            Err(_) => Err(Error::from_error_string(ERROR_DESERIALIZATION.to_string())),
        }
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
    fn from_str(data: &str) -> Result<Fighter, Error> {
        match serde_json::from_str(data) {
            Ok(object) => Ok(object),
            Err(_) => Err(Error::from_error_string(ERROR_DESERIALIZATION.to_string())),
        }
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
    fn from_str(data: &str) -> Result<Fight, Error> {
        match serde_json::from_str(data) {
            Ok(object) => Ok(object),
            Err(_) => Err(Error::from_error_string(ERROR_DESERIALIZATION.to_string())),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ErrorDetail {
    pub r#type: Option<ErrorType>,
    pub message: String,
}

impl MunModel for ErrorDetail {
    fn from_str(data: &str) -> Result<ErrorDetail, Error> {
        match serde_json::from_str(data) {
            Ok(object) => Ok(object),
            Err(_) => Err(Error::from_error_string(ERROR_DESERIALIZATION.to_string())),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Error {
    pub code: Option<u16>,
    pub detail: ErrorDetail,
}

impl Error {
    pub fn from_error_string(error_string: String) -> Error {
        Error {
            code: None,
            detail: ErrorDetail {
                r#type: None,
                message: error_string,
            },
        }
    }
}
