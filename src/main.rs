use std::error::Error;
use std::env;

use mongodb::options::FindOptions;
use rocket::serde::json::Json;
use rocket::http::{Method, Status};
use rocket::{get, post, routes};
use rocket_cors::{CorsOptions, AllowedOrigins};

use rocket_jwt::jwt;
use serde::{
    Deserialize, 
    Serialize
};

use chrono::Local;

use mongodb::{
    bson::{ doc },
    bson,
    sync::{Client, Collection },
};

static SECRET_KEY:&str = env!("JWT_SIGNING_KEY");
const ANONYMOUS_ROLE:&str = "ANONYMOUS_USER";

#[jwt(SECRET_KEY)]
struct UserClaim {
    username: String,
    roles: Vec<String>
}

#[derive(Serialize, Deserialize)]
struct NewShit {
    username:String,
    text:String,
    date:String
}

#[derive(Serialize, Deserialize)]
struct ShitEntity {
    #[serde(rename = "_id")]
    id:bson::oid::ObjectId,
    username:String,
    text:String,
    date:String
}

#[derive(Serialize, Deserialize)]
struct ShitRO {
    id:String,
    username:String,
    text:String,
    date:String
}

#[derive(Serialize, Deserialize)]
struct ShitRequest {
    text: String
}

#[get("/")]
fn get_shits() -> Result<Json<Vec<ShitRO>>, Status> {
    let uri = match env::var("DB_URI") {
        Ok(v) => v.to_string(),
        Err(_) => format!("Error loading env variable"),
    };
    let client = Client::with_uri_str(uri).unwrap();
    let db = client.database("shitter-db");
    let col: Collection<ShitEntity> = db.collection("shits");

    let mut shits:Vec<ShitRO> = vec![];
    let cursor = col.find(doc!{}, FindOptions::builder().sort(doc!{"date": -1}).build()).expect("Failed to find shits");

    for result in cursor {
        let ShitEntity {id, username, text, date} = result.unwrap();
        shits.push(ShitRO {
            id: id.to_hex(),
            username,
            text,
            date
        });
    }

    return Ok(Json(shits));
}

#[post("/", data="<shit_request>")]
fn create_shit(shit_request: Json<ShitRequest>, user: UserClaim) -> Result<Json<ShitRO>, Status> {
    if user.roles.contains(&ANONYMOUS_ROLE.to_string()) {
        return Err(Status::Forbidden);
    }

    let uri = match env::var("DB_URI") {
        Ok(v) => v.to_string(),
        Err(_) => format!("Error loading env variable"),
    };
    let client = Client::with_uri_str(uri).unwrap();
    let db = client.database("shitter-db");
    let col: Collection<NewShit> = db.collection("shits");

    let new_shit:NewShit = NewShit {
        username: user.username.to_owned(),
        text: shit_request.text.to_owned(),
        date: Local::now().to_rfc3339()
    };

    let result = col
        .insert_one(&new_shit, None)
        .ok()
        .unwrap();

    let created_shit = ShitRO {
        id: result.inserted_id.as_object_id().unwrap().to_hex(),
        username: new_shit.username.to_owned(),
        text: new_shit.text.to_owned(),
        date: new_shit.date.to_owned()
    };

    return Ok(Json(created_shit));
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // You can also deserialize this
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);

    let _ = rocket::build()
        .mount("/shits", routes![get_shits, create_shit])
        .attach(cors.to_cors().unwrap())
        .attach(UserClaim::fairing())
        .launch()
        .await?;

    Ok(())
}