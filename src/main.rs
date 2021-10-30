#[macro_use] extern crate rocket;
use rocket::{State, serde::json::{Json, Value, json}, http::{Status, ContentType}, response::{Redirect}, uri};
pub mod utils;
use utils::{NewShort, Url, Key};
use futures::stream::{TryStreamExt};
use mongodb::{Client, Collection, bson::doc};
use dotenv;


// Main function
#[launch]
async fn launch() -> _ {
    dotenv::dotenv().ok();
    let check = check_env();
    if check {
        eprintln!("There are errors in your env file. See above output for more details.");
        eprintln!("Server will start in 2000ms");
        std::thread::sleep(std::time::Duration::from_millis(2000));
    }
    let connection = Client::with_uri_str(std::env::var("mongodb_uri").unwrap()).await.unwrap();
    let database = connection.database(&std::env::var("mongodb_db").unwrap());
    let collection = database.collection::<Url>(&std::env::var("mongodb_col").unwrap());
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
        <a href=\"https://github.com/matthewthechickenman/url-shortener\"><h4>url-shortener/0.0.1</h4></a>
    </body>
</html>"))
}

// Routes
#[put("/new", format = "json", data = "<data>")]
async fn new(collection: &State<Collection<Url>>, data: Json<NewShort>, key: Key) -> (Status, (ContentType, Option<Value>)) {
    if key != Key::from_string(std::env::var("key").unwrap()) {
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
    let doc = collection.inner().clone().find_one(doc! {"id": id}, None).await.unwrap();
    if doc.is_none() {
        println!("1{:?}", doc);
        return Redirect::to(uri!("/"));
    } else {
        println!("2{:?}", doc);
        let unwrapped = doc.unwrap();
        if std::env::var("track_clicks").unwrap().contains("1") {
            collection.inner().clone().update_one(doc! {"id":unwrapped.id}, doc! {"$inc": {"cl": 1}}, None).await.unwrap();
        }
        return Redirect::to(format!("//{}", unwrapped.to));
    }
}

#[get("/data/<id>")]
async fn data(collection: &State<Collection<Url>>, id: String, key: Key) -> (Status, (ContentType, Option<Value>)) {
    if key != Key::from_string(std::env::var("key").unwrap()) || key.as_string().len() <= 0 {
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
    if key != Key::from_string(std::env::var("key").unwrap()) {
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

// Utils
fn check_env() -> bool {
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
        Ok(_) => {},
        Err(_) => {
            std::env::set_var("track_clicks", "0");
            boolean = true;
            eprintln!("track_clicks has invalid value, setting to 0");
        }
    };
    return boolean;
}