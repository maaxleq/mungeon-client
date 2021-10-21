use crate::model;
use crate::net;

static ERROR_STATUS_UNINITALIZED: &'static str =
    "Error while accessing player status, status is uninitialized";

#[derive(Debug)]
pub enum ConnectionStatus {
    CONNECTED,
    CONNECTING,
    DISCONNECTED,
}

#[derive(Debug)]
pub struct Session<'a> {
    pub status: Option<model::Status>,
    pub client: net::MunHttpClient,
    pub error: Option<model::Error>,
    pub status_info: Vec<&'a str>,
    pub secondary_info: Vec<&'a str>,
}

impl<'a> Session<'a> {
    pub fn new(url: String) -> Session<'a> {
        Session {
            status: None,
            client: net::MunHttpClient::new(url),
            error: None,
            status_info: Vec::new(),
            secondary_info: Vec::new(),
        }
    }

    pub fn disconnect(&mut self) {
        self.status = None;
        self.clean();
    }

    pub fn is_connected(&self) -> bool {
        match self.status {
            Some(_) => true,
            None => false,
        }
    }

    pub fn update_room(&mut self, room: model::Room) {
        match &mut self.status {
            Some(status) => status.room = room,
            _ => (),
        }
    }

    pub fn get_guid(&self) -> String {
        match &self.status {
            Some(status) => status.guid.clone(),
            _ => panic!("{}", ERROR_STATUS_UNINITALIZED),
        }
    }

    pub fn clean(&mut self) {
        self.error = None;
        self.status_info.clear();
        self.secondary_info.clear();
    }

    pub fn connect(&mut self) {
        match self.client.connect() {
            Ok(status) => {
                self.status = Some(status.clone());
                self.status_info.push(format!("id: {}", status.guid).as_str());
                self.status_info.push(format!("life: {}", status.total_life).as_str());
                self.status_info.push(format!("room: {}", status.room.description).as_str());
                self.status_info.push("directions:");
                
            },
            Err(error) => self.error = Some(error),
        }
    }

    pub fn look_room(&mut self) {
        match self.client.look_room(self.get_guid()) {
            Ok(room) => self.update_room(room),
            Err(error) => self.error = Some(error),
        }
    }

    pub fn r#move(&mut self, direction: model::Direction) {
        match self.client.r#move(self.get_guid(), direction) {
            Ok(room) => self.update_room(room),
            Err(error) => self.error = Some(error),
        }
    }

    pub fn look_entity(&mut self, guid_dest: String) {
        match self.client.look_entity(self.get_guid(), guid_dest) {
            Ok(entity) => entity.append_to_info(&mut self.secondary_info),
            Err(error) => self.error = Some(error),
        }
    }

    pub fn attack(&mut self, guid_dest: String) {
        match self.client.attack(self.get_guid(), guid_dest) {
            Ok(fight) => fight.append_to_info(&mut self.secondary_info),
            Err(error) => self.error = Some(error),
        }
    }
}
