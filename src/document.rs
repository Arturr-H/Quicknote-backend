use std::collections::HashMap;

/*- Imports -*/
use serde::{ Serialize, Deserialize };

/*- Main doc strcut -*/
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Document {
    pub title: String,
    pub description: String,
    pub date: u64,

    /*- The document's content -*/
    pub texts: HashMap<String, Text>,
    pub notes: HashMap<String, Note>,
    pub canvases: HashMap<String, Canvas>,

    /*- The document's owner -*/
    pub owner: String,
    pub id: String,
}

/*- Doc attributes -*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Coordinate { x: i32, y: i32 }
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextSize { width: u32, height: u32, font_size:u32 }
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Size { width: u32, height: u32 }

/*- Text struct -*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Text {
    _real_content: Vec<u8>,
    position: Coordinate,
    size: TextSize
}

/*- Note struct -*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Note {
    position: Coordinate,
    size: Size,
    _real_content: Vec<u8>,
}

/*- Canvas struct -*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Canvas {
    position: Coordinate,
    id: String,
}

