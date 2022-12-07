use std::collections::HashMap;

/*- Imports -*/
use serde::{ Serialize, Deserialize };
use serde_json;

/*- Main doc strcut -*/
#[derive(Serialize, Deserialize, Debug)]
pub struct Document {
    pub title: String,
    pub description: String,

    /*- The document's content -*/
    pub texts: HashMap<String, Text>,
    pub notes: HashMap<String, Note>,
    pub canvases: HashMap<String, Canvas>,

    /*- The document's owner -*/
    pub owner: String,
}

/*- Doc attributes -*/
#[derive(Serialize, Deserialize, Debug)]
pub struct Coordinate { x: i32, y: i32 }
#[derive(Serialize, Deserialize, Debug)]
pub struct Scale { width: u32, height: u32 }

/*- Text struct -*/
#[derive(Serialize, Deserialize, Debug)]
pub struct Text {
    content: String,
    position: Coordinate,
    scale: Scale
}

/*- Note struct -*/
#[derive(Serialize, Deserialize, Debug)]
pub struct Note {
    content: String,
    position: Coordinate,
}

/*- Canvas struct -*/
#[derive(Serialize, Deserialize, Debug)]
pub struct Canvas {
    content: String,
    position: Coordinate,
}

