/*- Imports -*/
use serde::{ Serialize, Deserialize };
use serde_json::{self, json, Value};
use responder::prelude::*;
use reqwest::blocking::Client as ReqClient;
use mongodb::sync::{
    Client,
    Collection,
    Database
};

use crate::{ MONGO_CLIENT_URI_STRING, ACCOUNT_API_URL };

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SuidResponse {
    suid: String,
}

#[derive(Debug)]
pub(crate) enum AuthorizationStatus{
    Authorized(String),
    Unauthorized,
    Err,
}

/*- Fully check if user is authorized, and return
    `AuthorizationStatus` dependent on if they are -*/
pub(crate) fn authenticate(stream:&mut Stream) -> AuthorizationStatus {
    let cookies = stream.get_cookies();

    /*- Check if the user has a cookie -*/
    let token = match cookies.get("token") {
        Some(value) => value,
        None => {
            match stream.headers.get("token") {
                Some(value) => value,
                None => return AuthorizationStatus::Err,
            }
        },
    };
    let text = match match ReqClient::new()
        .get(&format!("{}profile/verify-token", &**ACCOUNT_API_URL))
        .header("token", *token)
        .send() {
            Ok(e) => e,
            Err(_) => return AuthorizationStatus::Err,
        }.text() {
            Ok(e) => e,
            Err(_) => return AuthorizationStatus::Err,
        };

    /*- Check if the user is authorized -*/
    match serde_json::from_str::<SuidResponse>(&text) {
        Ok(e) => {
            /*- Check if the user is authorized -*/
            if e.suid == "0" {
                return AuthorizationStatus::Unauthorized;
            } else {
                return AuthorizationStatus::Authorized(e.suid);
            }
        },
        Err(_) => AuthorizationStatus::Err
    }

}

/*- Quick way of establishing a connection with the mongo client -*/
pub(crate) fn establish_mclient<Type__>(collection_name:&str) -> Collection<Type__> {
    /*- Establish the mongodb connection -*/
    let client:Client = Client::with_uri_str(
        &**MONGO_CLIENT_URI_STRING
    ).expect("Failed to initialize standalone client.");

    /*- Get the database -*/
    let db:Database = client.database("documents");

    /*- Get the collection -*/
    let collection:Collection<Type__> = db.collection::<Type__>(collection_name);

    /*- Return the collection -*/
    collection
}

