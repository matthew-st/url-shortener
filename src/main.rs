#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;
pub mod utils;
use rocket::{State, serde::json::{Json, Value, json}, http::{Status, ContentType}, tokio::sync::Mutex};
use utils::{NewShort, Url, Key, check_env};
use futures::stream::{TryStreamExt};
use std::{collections::{HashMap}, thread, time::{Duration}, env, sync::{Arc}};
use mongodb::{Client, Collection, bson::doc};
use dotenv;

lazy_static! {
    static ref CACHE: Arc<Mutex<HashMap<String, i64>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref LINKS: Arc<Mutex<HashMap<String, Option<Url>>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref NOTFOUND_PAGE: String = "<head><title>404 Not Found</title></head><html><body style=\"text-align:center;width:100%;\"><h2>Nothing to see here!</h2><hr/><a href=\"https://github.com/matthewthechickenman/url-shortener\"><h4>url-shortener/1.1.1</h4></a></body></html>".to_string();
}

// Main function
#[launch]
async fn launch() -> _ {
    dotenv::dotenv().ok();
    let check = check_env();
    if check {
        eprintln!("There are errors in your env file. See above output for more details.");
        eprintln!("Server will start in 2 seconds.");
        thread::sleep(Duration::from_secs(2));
    }
    let connection = Client::with_uri_str(env::var("mongodb_uri").unwrap()).await.unwrap();
    let database = connection.database(&env::var("mongodb_db").unwrap());
    let collection = database.collection::<Url>(&env::var("mongodb_col").unwrap());
    let collection_2 = collection.clone();
    rocket::tokio::spawn(async move {
        loop {
            let mut clicks = CACHE.lock().await;
            for url in clicks.iter() {
                collection_2.update_one(doc!{"id": url.0}, doc!{"$inc": {"cl": url.1}}, None).await.unwrap();
            }
            clicks.clear();
            drop(clicks);
            thread::sleep(Duration::from_secs(env::var("save_after").unwrap().parse::<u64>().unwrap()));
        }
    });
    rocket::build()
    .mount("/", routes![redirect])
    .mount("/api", routes![new])
    .mount("/api", routes![data])
    .mount("/api", routes![all])
    .register("/", catchers![not_found])
    .manage(collection)
}


// Catchers
#[catch(404)]
fn not_found() -> (Status, (ContentType, String)) {
    (Status::NotFound, (ContentType::HTML, NOTFOUND_PAGE.to_string()))
}

// Routes
#[put("/new", format = "json", data = "<data>")]
async fn new(collection: &State<Collection<Url>>, data: Json<NewShort>, key: Key) -> (Status, (ContentType, Option<Value>)) {
    if key != Key::from_string(env::var("key").unwrap()) {
        return (Status::Forbidden, (ContentType::JSON, Some(json!({"error": true, "reason": "incorrect key"}))))
    } else if !data.url.contains("https://") && !data.url.contains("http://") {
        return (Status::BadRequest, (ContentType::JSON, Some(json!({"error": true, "reason": "bad request body"})))) 
    } else {
        let coll = collection.inner().clone();
        let url = String::from(&data.url).replace("https://", "").replace("http://", "");
        let res = Url::from_url(url.clone());
        let doc = coll.find_one(doc! {"to": url}, None).await.unwrap();
        if !doc.is_none() {
            return (
                Status::Ok, 
                (
                    ContentType::JSON,
                    Some(json!({"id": doc.clone().unwrap().id, "to": doc.unwrap().to}))
                ));
            } else {
                coll.insert_one(res.clone(), None).await.unwrap();
                LINKS.lock().await.insert(res.clone().id, Some(res.clone()));
                return (
                    Status::Ok, 
                    (
                        ContentType::JSON,
                        Some(json!({"id": res.id, "to": res.to}))
                    ));
                }
            }
        }
        
#[get("/<id>")]
async fn redirect(collection: &State<Collection<Url>>, id: String) -> (Status, (ContentType, String)) {
    let mut cached = LINKS.lock().await;
    if cached.get(&id).is_some() {
        if cached.get(&id).unwrap().as_ref().is_some() {
            let unwrapped = cached.get(&id).unwrap().as_ref().unwrap();
            return (Status::Ok, (ContentType::HTML, format!("<meta http-equiv=\"refresh\" content=\"0;url=https://{}\"/>", unwrapped.to)));
        } else {
            return (Status::NotFound, (ContentType::HTML, NOTFOUND_PAGE.to_string()));
        }
    } else if cached.get(&id).is_none() {
        return (Status::NotFound, (ContentType::HTML, NOTFOUND_PAGE.to_string()));
    }
    let doc = collection.inner().clone().find_one(doc! {"id": id.clone()}, None).await.unwrap();
    if doc.is_none() {
        cached.insert(id, None);
        return (Status::NotFound, (ContentType::HTML, NOTFOUND_PAGE.to_string()));
    } else {
        let unwrapped = doc.unwrap();
        if env::var("track_clicks").unwrap().contains("1") {
            let mut cache = CACHE.lock().await;
            let choice = *cache.get(&id.clone()).unwrap_or(&0);
            cache.insert(id, choice + &1);
        }
        return (Status::Ok, (ContentType::HTML, format!("<meta http-equiv=\"refresh\" content=\"0;url=https://{}\"/>", unwrapped.to)));
    }
}
        
#[get("/data/<id>")]
async fn data(collection: &State<Collection<Url>>, id: String, key: Key) -> (Status, (ContentType, Option<Value>)) {
    if key != Key::from_string(env::var("key").unwrap()) || key.as_string().len() <= 0 {
        return (Status::Forbidden, (ContentType::JSON, Some(json!({"error": true, "reason": "incorrect key"}))))
    }
    let document = collection.inner().clone().find_one(doc! {"id": id}, None).await.unwrap();
    if document.is_none() {
        return (Status::NotFound, (ContentType::JSON, Some(json!({"error": true, "reason": "not found"}))))
    }
    let doc = document.unwrap();
    let send = doc! {"id": doc.id.clone(), "to": doc.to, "cl": doc.cl + CACHE.lock().await.get(&doc.id).unwrap_or(&0)};
    (Status::Ok, 
        (
            ContentType::JSON,
            Some(json!(send))
        )
    )
}
        
#[get("/all")]
async fn all(collection: &State<Collection<Url>>, key: Key) -> (Status, (ContentType, Option<Value>)) {
    if key != Key::from_string(env::var("key").unwrap()) {
        return (Status::Forbidden, (ContentType::JSON, Some(json!({"error": true, "reason": "incorrect key"}))))
    }
    let all: Vec<Url> = collection.inner().clone().find(doc! {}, None).await.unwrap().try_collect().await.unwrap();
    (Status::Ok, 
        (
            ContentType::JSON,
            Some(json!(all))
        )
    )
}
        
        