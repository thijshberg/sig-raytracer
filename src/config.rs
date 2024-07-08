use jpeg_decoder::Decoder;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::fs::File;
use std::io::BufReader;

use crate::camera::Camera;
use crate::cube::Cube;


#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct Sky {
    // If provided, the sky will be rendered using the equirectangular
    // projected texture loaded from an image file at this path. Else,
    // a light blue colored sky will be used.
    #[serde_as(as = "TextureOptionPixelsAsPath")]
    pub texture: Option<(Vec<u8>, usize, usize, String)>,
}

impl Sky {
    pub fn new_default_sky() -> Sky {
        Sky { texture: None }
    }
}

fn load_texture_image(path: &str) -> (Vec<u8>, usize, usize, String) {
    let file = File::open(path).expect(path);
    let mut decoder = Decoder::new(BufReader::new(file));
    let pixels = decoder.decode().expect("failed to decode image");
    let metadata = decoder.info().unwrap();
    (
        pixels,
        metadata.width as usize,
        metadata.height as usize,
        path.to_string(),
    )
}

serde_with::serde_conv!(
    TextureOptionPixelsAsPath,
    Option<(Vec<u8>, usize, usize, String)>,
    |texture: &Option<(Vec<u8>, usize, usize, String)>| {
        match texture {
            Some(tuple) => tuple.3.clone(),
            None => "".to_string(),
        }
    },
    |value: &str| -> Result<_, std::convert::Infallible> {
        match value {
            "" => Ok(None),
            _ => Ok(Some(load_texture_image(value))),
        }
    }
);

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub width: usize,
    pub height: usize,
    pub samples_per_pixel: u32,
    pub max_depth: usize,
    pub sky: Option<Sky>,
    pub camera: Camera,
    pub objects: Vec<Cube>,
    pub nr_probes: i64,
}

