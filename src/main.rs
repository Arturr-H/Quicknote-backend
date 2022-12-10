/*- Global allowings -*/
#![allow(
    dead_code,
    unused_imports
)]

/*- Imports -*/
mod utils;
mod document;
use lazy_static::lazy_static;
use responder::prelude::*;
use dotenv::dotenv;
use document::Document;
use mongodb::{ self, bson::doc, };
use serde::{self, __private::doc};
use uuid;

/*- Constants -*/
lazy_static! {
    pub(crate) static ref MONGO_CLIENT_URI_STRING:String = std::env::var("MONGO_CLIENT_URI_STRING").unwrap();
    pub(crate) static ref ACCOUNT_API_URL:String = std::env::var("ACCOUNT_API_URL").unwrap();
    pub(crate) static ref JWT_SECRET_KEY:String = std::env::var("JWT_SECRET_KEY").unwrap();
}

/*- Initialize -*/
fn main() -> () {
    dotenv().unwrap();

    /*- Create enpoint routes -*/
    let routes = &[
        Route::Get("get-documents", get_docs),
        Route::Get("set-doc", set_doc),
        Route::Get("get-doc", get_doc),
        Route::Get("add-doc", add_doc),
    ];

    /*- Start the server -*/
    Server::new()
        .address("127.0.0.1")
        .port(8080)
        .routes(routes)
        .start().unwrap()
}

/*- API endpoints -*/
fn get_docs(stream: &mut Stream) -> () {
    /*- Authenticate the user -*/
    let suid = match utils::authenticate(stream) {
        utils::AuthorizationStatus::Authorized(suid) => suid,
        _ => {
            return stream.respond_status(401);
        }
    };

    /*- Establish mongodb client -*/
    let client = utils::establish_mclient::<Document>("documents");

    /*- Get the user's documents -*/
    let mut docs:Vec<Document> = Vec::new();
    for doc in client.find(doc! {
        "owner": &suid
    }, None).unwrap() {
        docs.push(doc.unwrap());
    };

    /*- Respond with the documents -*/
    stream.respond(200, Respond::new().json(
        match &serde_json::to_string(&docs) {
            Ok(e) => e,
            Err(_) => return stream.respond_status(500)
        }
    ));
}
fn set_doc(stream:&mut Stream) -> () {
    /*- Authenticate the user -*/
    let suid = match utils::authenticate(stream) {
        utils::AuthorizationStatus::Authorized(suid) => suid,
        _ => {
            return stream.respond_status(401);
        }
    };

    /*- Establish mongodb client -*/
    let client = utils::establish_mclient::<Document>("documents");

    /*- Get the user's doc -*/
    let mut document:Document = match serde_json::from_str(match stream.headers.get("document") {
        Some(e) => e,
        None => return stream.respond_status(400)
    }) {
        Ok(e) => e,
        Err(e) => panic!("{e}")
    };
    document.owner = suid.clone();

    /*- Check if the document already exists -*/
    if client.find_one(doc! {
        "owner": &suid,
        "id": &document.id
    }, None).unwrap_or(None).is_some() {
        /*- Update doc -*/
        match client.replace_one(doc! {
            "owner": &suid,
            "id": &document.id
        }, &document, None) {
            Ok(_) => (),
            Err(_) => return stream.respond_status(500)
        };
    }else {
        /*- Insert the document -*/
        client.insert_one(&document, None).unwrap();
    }

    /*- Respond with the documents -*/
    stream.respond(
        200,
        Respond::new()
            .json(&serde_json::to_string(&document).unwrap_or("".into()))
    );
}
fn get_doc(stream:&mut Stream) -> () {
    /*- Authenticate the user -*/
    let suid = match utils::authenticate(stream) {
        utils::AuthorizationStatus::Authorized(suid) => suid,
        _ => {
            return stream.respond_status(401);
        }
    };

    /*- Establish mongodb client -*/
    let client = utils::establish_mclient::<Document>("documents");

    /*- Get the user's document id -*/
    let id:&str = match stream.headers.get("id") {
        Some(e) => e,
        None => return stream.respond_status(400)
    };

    /*- Get the document -*/
    let doc = match match client.find_one(doc! { "owner": suid, "id": id }, None) {
        Ok(e) => e,
        Err(_) => return stream.respond_status(404)
    } {
        Some(e) => e,
        None => return stream.respond_status(404)
    };

    /*- Respond with the documents -*/
    stream.respond(200, Respond::new().json(
        match &serde_json::to_string(&doc) {
            Ok(e) => e,
            Err(_) => return stream.respond_status(500)
        },
    ));
}
fn add_doc(stream:&mut Stream) -> () {
    /*- Authenticate the user -*/
    let suid = match utils::authenticate(stream) {
        utils::AuthorizationStatus::Authorized(suid) => suid,
        _ => {
            return stream.respond_status(401);
        }
    };

    /*- Get metadata -*/
    let (title, description) = match (stream.headers.get("title"), stream.headers.get("description")) {
        (Some(title), Some(description)) => (title, description),
        (Some(title), None) => (title, &"No description provided"),
        (None, Some(description)) => (&"Unnamed document", description),
        (None, None) => (&"Unnamed document", &"No description provided")
    };
    println!("{title} {description}");

    /*- Establish mongodb client -*/
    let client = utils::establish_mclient::<Document>("documents");

    /*- Generate ID -*/
    let id = uuid::Uuid::new_v4().to_string();

    /*- Insert the document -*/
    let doc = &Document {
        id,
        owner: suid,
        title: title.to_string(),
        description: description.to_string(),
        ..Default::default()
    };
    client.insert_one(doc, None).unwrap();

    /*- Respond with the documents -*/
    stream.respond(200, Respond::new().json(
        match &serde_json::to_string(&doc) {
            Ok(e) => {println!("{e}");e},
            Err(_) => return stream.respond_status(500)
        },
    ));
}