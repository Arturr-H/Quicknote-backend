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
use serde;

/*- Constants -*/
lazy_static! {
    pub(crate) static ref MONGO_CLIENT_URI_STRING:String = std::env::var("MONGO_CLIENT_URI_STRING").unwrap();
    pub(crate) static ref ACCOUNT_API_URL:String = std::env::var("ACCOUNT_API_URL").unwrap();
    pub(crate) static ref JWT_SECRET_KEY:String = std::env::var("JWT_SECRET_KEY").unwrap();
}

/*- Structs, enums & unions -*/


/*- Initialize -*/
fn main() -> () {
    dotenv().unwrap();

    /*- Create enpoint routes -*/
    let routes = &[
        Route::Get("get-documents", get_docs),
        Route::Get("create-doc", create_doc),
        Route::Get("get-doc", get_doc)
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
    let docs = client.find(doc! { "owner": suid }, None).unwrap();
    for doc in docs {
        println!("{:?}", doc.unwrap().owner);
    }

    /*- Respond with the documents -*/
    stream.respond(200, Respond::new().headers(vec!["Access-Control-Allow-Origin: *".to_string()]));
}
fn create_doc(stream: &mut Stream) -> () {
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
        Err(_) => return stream.respond_status(400)
    };
    document.owner = suid;

    /*- Insert the document -*/
    client.insert_one(document, None).unwrap();

    /*- Respond with the documents -*/
    stream.respond(200, Respond::new().headers(vec!["Access-Control-Allow-Origin: *".to_string()]));
}
fn get_doc(stream:&mut Stream) -> () {
    println!("get-doc {:?}", stream.get_cookies());
    /*- Authenticate the user -*/
    let suid = match utils::authenticate(stream) {
        utils::AuthorizationStatus::Authorized(suid) => suid,
        _ => {
            return stream.respond_status(401);
        }
    };

    /*- Establish mongodb client -*/
    let client = utils::establish_mclient::<Document>("documents");

    /*- Get the user's document title -*/
    let title:&str = match stream.headers.get("title") {
        Some(e) => e,
        None => return stream.respond_status(400)
    };

    /*- Get the document -*/
    let doc = match match client.find_one(doc! { "owner": suid, "title": title }, None) {
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
    ).headers(vec!["Access-Control-Allow-Origin: *".to_string()]));
}