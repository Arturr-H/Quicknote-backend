/*- Global allowings -*/
#![allow(
    dead_code,
    unused_imports,
    unused_variables
)]

/*- Imports -*/
mod utils;
mod document;
use lazy_static::lazy_static;
use responder::prelude::*;
use dotenv::dotenv;
use document::Document;
use mongodb::{ self, bson::doc };
use uuid;
use std::{ fs, io::Write };

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
        Route::Get("set-doc",       set_doc),
        Route::Get("get-doc",       get_doc),
        Route::Get("add-doc",       add_doc),
        Route::Get("delete-doc",    delete_doc),
        Route::Post("save-canvas",  save_canvas),
        Route::Post("save-note",    save_note),
    ];

    /*- Start the server -*/
    Server::new()
        .address("0.0.0.0")
        .port(8081)
        .routes(routes)
        .init_buf_size(1024 /*- 1kb -*/ * 1024 /*- 1mb -*/ * 10 /*- 10mb -*/)
        .threads(10)
        .serve("./uploads/")
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
    for doc in match client.find(doc! {
        "owner": &suid
    }, None) {
        Ok(e) => e,
        Err(_) => return stream.respond_status(500)
    } {
        docs.push(match doc {
            Ok(e) => e,
            Err(_) => continue
        });
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
        None => panic!("a")
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
        match client.insert_one(&document, None) {
            Ok(_) => (),
            Err(_) => return stream.respond_status(500)
        };
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
fn delete_doc(stream:&mut Stream) -> () {
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
    let doc = match match client.find_one(doc! { "owner": &suid, "id": id }, None) {
        Ok(e) => e,
        Err(_) => return stream.respond_status(404)
    } {
        Some(e) => e,
        None => return stream.respond_status(404)
    };

    /*- Delete all canvases coupled to this doc -*/
    for canvas in &doc.canvases {
        let path = format!("canvases/{}-{}", id, canvas.1.id);

        /*- Delete the canvas file -*/
        std::fs::remove_file(path).ok();
    }

    /*- Delete the document -*/
    match client.delete_one(doc! {
        "owner": &suid,
        "id": id
    }, None) {
        Ok(_) => (),
        Err(_) => return stream.respond_status(500)
    };

    /*- Respond with the documents -*/
    stream.respond_status(200);
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
    match client.insert_one(doc, None) {
        Ok(_) => (),
        Err(_) => return stream.respond_status(500)
    };

    /*- Respond with the documents -*/
    stream.respond(200, Respond::new().json(
        match &serde_json::to_string(&doc) {
            Ok(e) => e,
            Err(_) => return stream.respond_status(500)
        },
    ));
}
fn save_canvas(stream:&mut Stream) -> () {
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
    let doc_id:&str = match stream.headers.get("doc-id") {
        Some(e) => e,
        None => return stream.respond_status(400)
    };
    let canvas_id:&str = match stream.headers.get("canvas-id") {
        Some(e) => e,
        None => return stream.respond_status(400)
    };

    /*- Get the canvas -*/
    let canvas = &stream.body;

    /*- Write canvas to file -*/
    let strpath = format!("uploads/canvases/{doc_id}-{canvas_id}");
    if canvas.len() != 0 {
        let mut file = match std::fs::File::create(strpath) {
            Ok(file) => file,
            Err(_) => return stream.respond_status(500)
        };
        let bytes_written = match file.write_all(canvas.as_bytes()) {
            Ok(_) => (),
            Err(_) => return stream.respond_status(500)
        };
    };

    stream.respond_status(200);
}
fn save_note(stream:&mut Stream) -> () {
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
    let doc_id:&str = match stream.headers.get("doc-id") {
        Some(e) => e,
        None => return stream.respond_status(400)
    };
    let note_id:&str = match stream.headers.get("note-id") {
        Some(e) => e,
        None => return stream.respond_status(400)
    };

    /*- Get the note -*/
    let note = &stream.body;

    /*- Write note to file -*/
    let strpath = format!("uploads/notes/{doc_id}-{note_id}");
    if note.len() != 0 {
        let mut file = match std::fs::File::create(strpath) {
            Ok(file) => file,
            Err(e) => panic!("{e}")
        };
        let bytes_written = match file.write_all(note.as_bytes()) {
            Ok(_) => (),
            Err(_) => return stream.respond_status(500)
        };
    };

    stream.respond_status(200);
}


