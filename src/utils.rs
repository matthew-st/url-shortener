use rocket::{serde::{Deserialize, Serialize}, Request, request::{self, FromRequest, Outcome}};
use nanoid::nanoid;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(crate="rocket::serde")]
pub struct Url {
    pub id: String,
    pub to: String,
    pub cl: i64
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

fn track_clicks_check(value: &String) -> bool {
    if value.ne("0") && value.ne("1") {
        return false;
    }
    return true;
}

pub fn check_env() -> bool {
    let mut boolean = false;
    std::env::var("mongodb_uri").expect("mongodb_uri not set in .env file");
    std::env::var("mongodb_db").expect("mongodb_db not set in .env file");
    let col = std::env::var("mongodb_col");
    match col {
        Ok(_) => {},
        Err(_) => {
            std::env::set_var("mongodb_col", "urls");
            boolean = true;
            eprintln!("mongodb_col has invalid value, setting to urls");
        }
    };
    std::env::var("key").expect("key not set in .env file");
    let track = std::env::var("track_clicks");
    match track {
        Ok(value) if track_clicks_check(&value) => {
            
        },
        _ => {
            std::env::set_var("track_clicks", "0");
            boolean = true;
            eprintln!("track_clicks has invalid value, setting to 0");
        }
    };
    let save_after = std::env::var("save_after");
    match save_after {
        Ok(value) if value.parse::<u64>().is_ok() => {
            
        },
        _ => {
            std::env::set_var("save_after", "60");
            boolean = true;
            eprintln!("save_after has invalid value, setting to 60");
        },
    };
    boolean
}

pub async fn save_clicks() {
    let clicks = CACHE.lock().unwrap();
            for url in clicks.iter() {
                collection_2.update_one(doc!{"id": url.0}, doc!{"$inc": {"cl": url.1}}, None).await.unwrap();
            }
    thread::sleep(Duration::from_secs(env::var("save_after").unwrap().parse::<u64>().unwrap()));
}