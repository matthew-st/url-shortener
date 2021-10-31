#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;
pub mod utils;
use rocket::{State, serde::json::{Json, Value, json}, http::{Status, ContentType}, response::{Redirect}, uri};
use utils::{NewShort, Url, Key, check_env};
use futures::stream::{TryStreamExt};
use std::{collections::{HashMap}, thread, time::{Duration}, env, sync::{Mutex}};
use mongodb::{Client, Collection, bson::doc};
use dotenv;

lazy_static! {
    static ref CACHE: Mutex<HashMap<String, i64>> = Mutex::new(HashMap::new());
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
            let clicks = CACHE.lock().unwrap();
            for url in clicks.iter() {
                collection_2.update_one(doc!{"id": url.0}, doc!{"$inc": {"cl": url.1}}, None).await.unwrap();
            }
            thread::sleep(Duration::from_secs(env::var("save_after").unwrap().parse::<u64>().unwrap()));
        }
    });
    rocket::build()
    .mount("/l/", routes![redirect])
    .mount("/api", routes![new])
    .mount("/api", routes![data])
    .mount("/api", routes![all])
    .register("/", catchers![not_found])
    .manage(collection)
}


// Catchers
#[catch(404)]
fn not_found() -> (Status, (ContentType, &'static str)) {
    (Status::NotFound, (ContentType::HTML, "<head>
    <title>404 Not Found</title>
    </head>
    <html>
    <body style=\"text-align:center;width:100%;\">
    <h2>Nothing to see here!</h2>
    <hr/>
    <a href=\"https://github.com/matthewthechickenman/url-shortener\"><h4>url-shortener/1.1.0</h4></a>
    </body>
    </html>"))
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
async fn redirect(collection: &State<Collection<Url>>, id: String) -> Redirect {
    let doc = collection.inner().clone().find_one(doc! {"id": id.clone()}, None).await.unwrap();
    if doc.is_none() {
        return Redirect::to(uri!("/"));
    } else {
        let unwrapped = doc.unwrap();
        if env::var("track_clicks").unwrap().contains("1") {
            let cache = CACHE.lock().unwrap();
            let choice = cache.get(&id.clone()).unwrap_or(&0);
            CACHE.lock().unwrap().insert(id, choice + &1);
        }
        return Redirect::to(format!("//{}", unwrapped.to));
    }
}
        
#[get("/data/<id>")]
async fn data(collection: &State<Collection<Url>>, id: String, key: Key) -> (Status, (ContentType, Option<Value>)) {
    if key != Key::from_string(env::var("key").unwrap()) || key.as_string().len() <= 0 {
        return (Status::Forbidden, (ContentType::JSON, Some(json!({"error": true, "reason": "incorrect key"}))))
    }
    let doc = collection.inner().clone().find_one(doc! {"id": id}, None).await.unwrap();
    (Status::Ok, 
        (
            ContentType::JSON,
            Some(json!(doc))
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
        
        