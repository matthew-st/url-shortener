use rocket::{serde::{Deserialize, Serialize}, Request, request::{self, FromRequest, Outcome}};
use nanoid::nanoid;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(crate="rocket::serde")]
pub struct Url {
    pub id: String,
    pub to: String,
    pub cl: u64
}

impl Url {
    pub fn from_url(url: String) -> Url {
        Url {id: nanoid!(10), to: url, cl: 0}
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(crate="rocket::serde")]
pub struct NewShort {
    pub url: String
}

#[derive(std::cmp::PartialEq)]
pub struct Key (String);

#[derive(Debug)]
pub enum KeyError {
    Missing,
    Invalid,
    TooMany
}

impl Key {
    pub fn from_string(string: String) -> Key {
        Key(string)
    }
    pub fn as_string(&self) -> String {
        return self.0.clone();
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Key {
    type Error = KeyError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let keys: Vec<&str> = request.headers().get("x-api-key").collect();
        match keys.len() {
            0 => Outcome::Success(Key("".to_string())),
            1 => Outcome::Success(Key(keys[0].to_string())),
            _ => Outcome::Success(Key(keys[0].to_string())),
        }
    }
}