use std::fs;

use clap::Parser;
use raytracer::config::Config;
use raytracer::raytracer::render;
use raytracer::signal_map::generate_sigmap;

#[derive(Parser)]
struct Args{
    config: String,
    ouput_filename: String,
    #[arg(long)]
    angles: bool,
    #[arg(long)]
    times: bool,
    #[arg(long)]
    png: bool,
    #[arg(long)]
    no_render: bool
}

fn main() {
    let args = Args::parse();
    let json = fs::read(&args.config).expect("Unable to read config file.");
    let scene = serde_json::from_slice::<Config>(&json).expect("Unable to parse config json");

    let filename = args.ouput_filename.as_str(); //format!("{}_{:0>3}.png", args[2], i);
    println!("\nRendering {}", filename);
    generate_sigmap(filename, &scene,args.times,args.angles,args.png);
    if !args.no_render{
        let view_name = filename.to_string() + "_view.png";
        render(&view_name,&scene);
    }
}
