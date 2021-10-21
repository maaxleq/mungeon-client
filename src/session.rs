use crate::model;
use crate::net;

static ERROR_STATUS_UNINITALIZED: &'static str =
    "Error while accessing player status, status is uninitialized";

#[derive(Debug)]
pub struct Session {
    pub status: Option<model::Status>,
    pub client: net::MunHttpClient,
    pub error: Option<model::Error>,
    pub status_info: Vec<String>,
    pub secondary_info: Vec<String>,
}

impl Session {
    pub fn new(url: String) -> Session {
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

    fn update_status_info(&mut self){
        match &self.status {
            Some(status) => {
                self.status_info.push(format!("<life> {}", status.total_life));
                self.status_info.push(format!("<room> {}", status.room.description));
                self.status_info.push(format!("<paths> {}", status.get_paths_string()));
                self.status_info.push(String::from("<entities>"));
                for entity in status.room.entities.iter() {
                    self.status_info.push(entity.clone());
                }
            },
            None => ()
        }
    }

    pub fn connect(&mut self) {
        match self.client.connect() {
            Ok(status) => {
                self.status = Some(status);
                self.update_status_info();
            },
            Err(error) => self.error = Some(error),
        }
    }

    pub fn look_room(&mut self) {
        match self.client.look_room(self.get_guid()) {
            Ok(room) => {
                self.update_room(room);
                self.update_status_info();
            },
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
            Ok(entity) => (),
            Err(error) => self.error = Some(error),
        }
    }

    pub fn attack(&mut self, guid_dest: String) {
        match self.client.attack(self.get_guid(), guid_dest) {
            Ok(fight) => (),
            Err(error) => self.error = Some(error),
        }
    }
}
