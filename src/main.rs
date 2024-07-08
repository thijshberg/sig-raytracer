use std::env;
use std::fs;

use raytracer::config::Config;
use raytracer::raytracer::render;
use raytracer::signal_map::generate_sigmap;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <config_file> <output_file>", args[0]);
        return;
    }

    let json = fs::read(&args[1]).expect("Unable to read config file.");
    let scene = serde_json::from_slice::<Config>(&json).expect("Unable to parse config json");

    let filename = args[2].as_str(); //format!("{}_{:0>3}.png", args[2], i);
    let mut angles:bool= false;
    let mut times:bool = false;
    let mut png:bool = false;
    for arg in &args[3..]{
        if arg == "--angles"{
            angles= true;
        }
        if arg == "--times"{
            times= true;
        }
        if arg == "--png"{
            png= true;
        }

    }
    println!("\nRendering {}", filename);
    generate_sigmap(&filename, &scene,times,angles,png);
    let view_name = filename.to_string() + "_view.png";
    render(&view_name,&scene);
}
