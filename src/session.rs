use crate::model;
use crate::net;

use std::collections::HashMap;

static ERROR_STATUS_UNINITALIZED: &'static str =
    "Error while accessing player status, status is uninitialized";

pub type EntityMap = HashMap<u32, String>;

#[derive(Clone, Debug)]
pub struct Session {
    pub status: Option<model::Status>,
    pub client: net::MunHttpClient,
    pub error: Option<model::Error>,
    pub fight_info: Option<model::Fight>,
    pub entity_info: Option<model::Entity>,
    pub entity_map: EntityMap,
}

impl Session {
    pub fn new(url: String) -> Session {
        Session {
            status: None,
            client: net::MunHttpClient::new(url),
            error: None,
            fight_info: None,
            entity_info: None,
            entity_map: EntityMap::new(),
        }
    }

    pub fn clear_entities(&mut self) {
        self.entity_map.clear();
    }

    pub fn update_entity_map(&mut self) {
        self.clear_entities();

        match &self.status {
            Some(status) => {
                let mut cpt: u32 = 1;

                for entity in status.room.entities.iter() {
                    let guid = entity.clone();

                    if guid != status.guid {
                        self.entity_map.insert(cpt, guid);
                        cpt += 1;
                    }
                }
            }
            None => (),
        }
    }

    pub fn get_entity_guid(&mut self, target_key: u32) -> Option<String> {
        for (key, val) in self.entity_map.iter() {
            if key == &target_key {
                return Some(val.clone());
            }
        }

        None
    }

    pub fn get_entities_keys(&self) -> Vec<u32> {
        let mut keys: Vec<u32> = Vec::new();

        for key in self.entity_map.keys() {
            keys.push(key.clone());
        }

        keys
    }

    pub fn disconnect(&mut self) {
        self.status = None;
        self.clear();
    }

    pub fn is_connected(&self) -> bool {
        match self.status {
            Some(_) => true,
            None => false,
        }
    }

    pub fn update_room(&mut self, room: model::Room) {
        match &mut self.status {
            Some(status) => {
                status.room = room;
                self.update_entity_map();
            }
            _ => (),
        }
    }

    pub fn get_guid(&self) -> Result<String, model::Error> {
        match &self.status {
            Some(status) => Ok(status.guid.clone()),
            _ => Err(model::Error {
                code: None,
                detail: model::ErrorDetail {
                    r#type: None,
                    message: ERROR_STATUS_UNINITALIZED.to_string(),
                },
            }),
        }
    }

    pub fn clear_infos(&mut self) {
        self.error = None;
        self.fight_info = None;
        self.entity_info = None;
    }

    fn clear(&mut self) {
        self.status = None;

        self.clear_entities();
        self.clear_infos();
    }

    pub fn connect(&mut self) {
        match self.client.connect() {
            Ok(status) => {
                self.status = Some(status);
                self.update_entity_map();
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub fn look_room(&mut self) {
        match self.get_guid() {
            Ok(guid) => match self.client.look_room(guid) {
                Ok(room) => {
                    self.update_room(room);
                }
                Err(error) => self.error = Some(error),
            },
            Err(error) => self.error = Some(error),
        }
    }

    pub fn r#move(&mut self, direction: model::Direction) {
        match self.get_guid() {
            Ok(guid) => match self.client.r#move(guid, direction) {
                Ok(room) => self.update_room(room),
                Err(error) => self.error = Some(error),
            },
            Err(error) => self.error = Some(error),
        }
    }

    pub fn look_entity(&mut self, guid_dest: String) {
        match self.get_guid() {
            Ok(guid) => match self.client.look_entity(guid, guid_dest) {
                Ok(entity) => self.entity_info = Some(entity),
                Err(error) => self.error = Some(error),
            },
            Err(error) => self.error = Some(error),
        }
    }

    pub fn attack(&mut self, guid_dest: String) {
        match self.get_guid() {
            Ok(guid) => match self.client.attack(guid, guid_dest) {
                Ok(fight) => self.fight_info = Some(fight),
                Err(error) => self.error = Some(error),
            },
            Err(error) => self.error = Some(error),
        }
    }
}
