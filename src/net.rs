use crate::model;
use crate::model::MunModel;

static ERROR_NETWORK: &'static str = "A network error has occured";
static ERROR_400: &'static str = "Bad request";
static ERROR_404: &'static str = "URL not found";
static ERROR_SERDE: &'static str = "Error while parsing JSON response";

use std::time;

#[derive(Clone, Debug)]
pub enum MunRequest {
    Get(String),
    Post(String, String),
}

#[derive(Clone, Debug)]
pub struct MunHttpClient {
    pub base_url: String,
    http_client: reqwest::blocking::Client,
    tried_once: bool
}

impl MunHttpClient {
    pub fn new(base_url: String) -> MunHttpClient {
        MunHttpClient {
            base_url: base_url,
            tried_once: false,
            http_client: reqwest::blocking::ClientBuilder::new()
                .timeout(time::Duration::from_secs(5))
                .build()
                .unwrap(),
        }
    }

    fn send_request<T>(&mut self, request: MunRequest) -> Result<T, model::Error>
    where
        T: model::MunModel,
    {
        let result = match request.clone() {
            MunRequest::Get(url) => self.http_client.get(url).send(),
            MunRequest::Post(url, body) => self
                .http_client
                .post(url)
                .body(body)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .send(),
        };

        match result {
            Ok(response) => {
                self.tried_once = false;
                match response.status().as_u16() {
                    400 => Err(model::Error {
                        code: Some(400),
                        detail: model::ErrorDetail {
                            r#type: None,
                            message: ERROR_400.to_string(),
                        },
                    }),
                    404 => Err(model::Error {
                        code: Some(404),
                        detail: model::ErrorDetail {
                            r#type: None,
                            message: ERROR_404.to_string(),
                        },
                    }),
                    409 => Err(match response.text() {
                        Ok(text) => model::Error {
                            code: Some(409),
                            detail: model::ErrorDetail::from_str(text.as_str())?,
                        },
                        Err(_) => Err(model::Error::from_error_string(ERROR_SERDE.to_string()))?,
                    }),
                    _ => match response.text() {
                        Ok(text) => T::from_str(text.as_str()),
                        Err(_) => Err(model::Error::from_error_string(ERROR_SERDE.to_string())),
                    },
                }
            },
            Err(_) => {
                if self.tried_once {
                    self.tried_once = false;
                    Err(model::Error {
                        code: None,
                        detail: model::ErrorDetail {
                            r#type: None,
                            message: ERROR_NETWORK.to_string(),
                        },
                    })
                }
                else {
                    self.tried_once = true;
                    self.send_request(request)
                }
            },
        }
    }

    pub fn connect(&mut self) -> Result<model::Status, model::Error> {
        let request = MunRequest::Post(format!("{}/connect", self.base_url), String::new());
        self.send_request::<model::Status>(request)
    }

    pub fn look_room(&mut self, guid: String) -> Result<model::Room, model::Error> {
        let request = MunRequest::Get(format!("{}/{}/regarder", self.base_url, guid));
        self.send_request::<model::Room>(request)
    }

    pub fn r#move(
        &mut self,
        guid: String,
        direction: model::Direction,
    ) -> Result<model::Room, model::Error> {
        let request = MunRequest::Post(
            format!("{}/{}/deplacement", self.base_url, guid),
            direction.to_movement_json(),
        );
        self.send_request::<model::Room>(request)
    }

    pub fn look_entity(
        &mut self,
        guid: String,
        guid_dest: String,
    ) -> Result<model::Entity, model::Error> {
        let request = MunRequest::Get(format!("{}/{}/examiner/{}", self.base_url, guid, guid_dest));
        self.send_request::<model::Entity>(request)
    }

    pub fn attack(&mut self, guid: String, guid_dest: String) -> Result<model::Fight, model::Error> {
        let request = MunRequest::Post(
            format!("{}/{}/taper/{}", self.base_url, guid, guid_dest),
            String::new(),
        );
        self.send_request::<model::Fight>(request)
    }
}
