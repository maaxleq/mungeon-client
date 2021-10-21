use crate::model;
use crate::model::MunModel;

static ERROR_REQWEST: &'static str = "A Reqwest error has occured";
static ERROR_400: &'static str = "Error 404: URL not found";
static ERROR_404: &'static str = "Error 400: Bad request";
static ERROR_SERDE: &'static str = "Error while parsing JSON response";

pub enum MunRequest {
    Get(String),
    Post(String, String),
}

#[derive(Debug)]
pub struct MunHttpClient {
    pub base_url: String,
    http_client: reqwest::blocking::Client,
}

impl MunHttpClient {
    pub fn new(base_url: String) -> MunHttpClient {
        MunHttpClient {
            base_url: base_url,
            http_client: reqwest::blocking::Client::new(),
        }
    }

    fn send_request<T>(&self, request: MunRequest) -> Result<T, model::Error>
    where
        T: model::MunModel,
    {
        let result = match request {
            MunRequest::Get(url) => self.http_client.get(url).send(),
            MunRequest::Post(url, body) => self.http_client.post(url).body(body).send(),
        };

        match result {
            Ok(response) => match response.status().as_u16() {
                400 => Err(model::Error {
                    code: 400,
                    detail: model::ErrorDetail {
                        r#type: None,
                        message: ERROR_400.to_string(),
                    },
                }),
                404 => Err(model::Error {
                    code: 404,
                    detail: model::ErrorDetail {
                        r#type: None,
                        message: ERROR_404.to_string(),
                    },
                }),
                409 => Err(model::Error {
                    code: 409,
                    detail: model::ErrorDetail::from_str(response.text().expect(ERROR_SERDE).as_str()),
                }),
                _ => Ok(T::from_str(response.text().expect(ERROR_SERDE).as_str())),
            },
            Err(_) => Err(model::Error {
                code: 404,
                detail: model::ErrorDetail {
                    r#type: None,
                    message: ERROR_REQWEST.to_string(),
                },
            }),
        }
    }

    pub fn connect(&self) -> Result<model::Status, model::Error> {
        let request = MunRequest::Post(format!("{}/connect", self.base_url), String::new());
        self.send_request::<model::Status>(request)
    }

    pub fn look_room(&self, guid: String) -> Result<model::Room, model::Error> {
        let request = MunRequest::Get(format!("{}/{}/regarder", self.base_url, guid));
        self.send_request::<model::Room>(request)
    }

    pub fn r#move(&self, guid: String, direction: model::Direction) -> Result<model::Room, model::Error> {
        let request = MunRequest::Post(format!("{}/{}/deplacement", self.base_url, guid), direction.to_movement_json());
        self.send_request::<model::Room>(request)
    }

    pub fn look_entity(&self, guid: String, guid_dest: String) -> Result<model::Entity, model::Error> {
        let request = MunRequest::Get(format!("{}/{}/examiner/{}", self.base_url, guid, guid_dest));
        self.send_request::<model::Entity>(request)
    }

    pub fn attack(&self, guid: String, guid_dest: String) -> Result<model::Fight, model::Error> {
        let request = MunRequest::Post(format!("{}/{}/taper/{}", self.base_url, guid, guid_dest), String::new());
        self.send_request::<model::Fight>(request)
    }
}
