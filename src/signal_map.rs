use byteorder::LittleEndian;
use byteorder::WriteBytesExt; // This trait adds methods to writeable types
use image::png::PNGEncoder;
use image::ColorType;
use palette::Pixel;
use palette::Srgb;
use rayon::prelude::*;
use std::fs::File;
use std::time::Instant;

use crate::config::Config;
use crate::cube::Cube;
use crate::materials::Material;
use crate::materials::Scatterable;
use crate::point3d::Point3D;
use crate::ray::HitRecord;
use crate::ray::Hittable;
use crate::ray::Ray;

fn find_lights(world: &Vec<Cube>) -> Vec<Cube> {
	world
		.iter()
		.filter(|s| match s.material {
			Material::Light(_) => true,
			_ => false,
		})
		.cloned()
		.collect()
}

fn write_image(
	filename: &str,
	pixels: &[u8],
	bounds: (usize, usize),
) -> Result<(), std::io::Error> {
	let output = File::create(filename)?;
	let encoder = PNGEncoder::new(output);
	encoder.encode(pixels, bounds.0 as u32, bounds.1 as u32, ColorType::RGB(8))?;
	Ok(())
}

fn hit_world<'material>(
	world: &'material Vec<&Cube>,
	r: &Ray,
	t_min: f32,
	t_max: f32,
) -> Option<HitRecord<'material>> {
	let mut closest_so_far = t_max;
	let mut hit_record = None;
	for obj in world {
		if let Some(hit) = obj.hit(r, t_min, closest_so_far) {
			//println!("Hit at {}",hit.point.y());
			closest_so_far = hit.t;
			hit_record = Some(hit);
		}
	}
	hit_record
}

fn clamp64(value: f32) -> f32 {
	if value < 0.0 {
		0.0
	} else if value > 1.0 {
		1.0
	} else {
		value
	}
}

fn coord_to_angle(x: f32, y: f32) -> f32 {
	if y < 0.0 {
		-x.acos()
	} else {
		x.acos()
	}
}

const BEAM_FALLOFF: f32 = 50.0;
const SUBSAMPLING: usize = 2;
fn rays_to(
	origin: Point3D,
	x: usize,
	y: usize,
	beams: i64,
	base_strength: f32,
	frequency: i64,
) -> Vec<Ray> {
	let mut res: Vec<Ray> = vec![];
	let interval = 1.0 / (SUBSAMPLING as f32);
	for i in 0..SUBSAMPLING {
		for j in 0..SUBSAMPLING {
			let direction = (Point3D::new(
				x as f32 + (i as f32) * interval - 0.5,
				0.0,
				y as f32 + (j as f32) * interval - 0.5,
			) - origin)
				.unit_vector();
			let flat_factor = (direction.x().powi(2) + direction.z().powi(2)).sqrt();
			let angle = coord_to_angle(direction.x() / flat_factor, direction.z() / flat_factor);
			let beam_centricity = ((angle * (beams as f32)) % (2.0 * std::f32::consts::PI)).abs()
				/ std::f32::consts::PI;
			let correct_centricity = if beam_centricity > 1.0 {
				beam_centricity - 2.0
			} else {
				beam_centricity
			};
			let strength =
				(base_strength - BEAM_FALLOFF) + BEAM_FALLOFF * (1.0 - correct_centricity.powi(2));
			res.push(Ray::new(origin, direction, strength, 0.0, frequency));
		}
	}
	res
}

//static DOWN: Point3D = Point3D::new(0.0, -1.0, 0.0);
fn generate_signal(
	station: &Cube,
	scene: &Config,
	do_times: bool,
	do_angles: bool,
) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
	let mut signals = vec![-140.0; (scene.width + 1) * (scene.height + 1)];
	let mut times = if do_times {
		vec![0.0; (scene.width + 1) * (scene.height + 1)]
	} else {
		vec![]
	};
	let mut angles = if do_times {
		vec![0.0; (scene.width + 1) * (scene.height + 1)]
	} else {
		vec![]
	};
	let objects: Vec<&Cube> = scene
		.objects
		.iter()
		.filter(|x| match x.material {
			Material::Light(_) => return false,
			_ => return true,
		})
		.collect();
	let (beams, base_strength, frequency) = match station.material {
		Material::Light(l) => Ok((l.beams, l.strength, l.frequency)),
		_ => Err(""),
	}
	.expect("Station does not have light material");
	//for i in 0..scene.nr_probes {
	for target_x in 0..scene.width {
		for target_y in 0..scene.height {
			assert!(base_strength > -130.0);
			for mut ray in rays_to(
				station.origin,
				target_x,
				target_y,
				beams,
				base_strength,
				frequency,
			) {
				//println!("Launching ray at {:?}",direction);
				for _i in 0..scene.max_depth {
					if let Some(hit_record) = hit_world(&objects, &ray, 0.00001, std::f32::MAX) {
						if hit_record.point.y() < 0.001
							&& hit_record.point.x() < scene.width as f32
							&& hit_record.point.z() < scene.height as f32
						{
							//We hit the ground, record the signal
							let x = hit_record.point.x() as usize;
							let y = hit_record.point.z() as usize;
							if ray.strength < base_strength {
								//println!( "hit the ground! {:?} = ({},{}), {} at {}", hit_record.point, x, y, ray.strength_at(hit_record.t), hit_record.t);
							}
							let coord = x + y * scene.width;
							if coord > signals.len() {
								println!(
									"Got out of bounds coordinate {} {} ({} {})!",
									x, y, scene.width, scene.height
								);
							} else if signals[coord] < ray.strength_at(hit_record.t) {
								signals[coord] = ray.strength_at(hit_record.t);
								if do_times {
									times[coord] = ray.ray_time + hit_record.t;
								}
								if do_angles {
									angles[coord] = ((hit_record.point.x() - x as f32)
										/ (hit_record.point.z() - y as f32))
										.atan();
								}
							}
						}
						match hit_record.material.scatter(&ray, &hit_record) {
							Some((possibly_new_ray, _)) => match possibly_new_ray {
								Some(new_ray) => {
									if hit_record.point.y() > 0.01 {
										//println!("Scattering at {} {}!",ray.origin.distance(&new_ray.origin),hit_record.t);
									}
									ray = new_ray;
								}
								None => {
									break;
								}
							},
							None => {
								//println!("Ray did not scatter at {} {} {}",hit_record.point.x(),hit_record.point.y(),hit_record.point.z());
								break;
							}
						}
					}
				}
			}
		}
	}
	(signals, times, angles)
}

fn signal_to_color(signal: f32) -> (u8, u8, u8) {
	let value = clamp64((signal + 100.0) / 100.0) * 3.0;
	let color = Srgb::new(
		(value - 2.0).clamp(0.0, 1.0),
		(value - 1.0).clamp(0.0, 1.0),
		value.clamp(0.0, 1.0),
	);
	let pixel: [u8; 3] = color.into_format().into_raw();
	return (pixel[0], pixel[1], pixel[2]);
}

fn signal_to_pixels(signal: &[f32], pixels: &mut Vec<u8>, dim_x: usize, dim_y: usize) {
	//println!("{} {}", signal[dim_x/2 + dim_y*dim_y/2],signal[dim_x/2+1 + dim_y*dim_y/2]);
	for x in 0..dim_x {
		for y in 0..dim_y {
			let (r, g, b) = signal_to_color(signal[x + y * dim_x]);
			//println!("{} {}: {}", x, y, r);
			set_pixel(pixels, (dim_x, dim_y), (x, y), (r, g, b));
		}
	}
}

fn set_pixel(
	pixels: &mut Vec<u8>,
	(dim_x, dim_y): (usize, usize),
	(x, y): (usize, usize),
	(r, g, b): (u8, u8, u8),
) {
	let y = dim_y - y;
	pixels[3 * x + 3 * y * dim_x] = r;
	pixels[3 * x + 3 * y * dim_x + 1] = g;
	pixels[3 * x + 3 * y * dim_x + 2] = b;
}
fn draw_rectangle(
	pixels: &mut Vec<u8>,
	dim_x: usize,
	dim_y: usize,
	side_x: usize,
	side_y: usize,
	origin_x: usize,
	origin_y: usize,
) {
	println!(
		"Drawing rectangle with origin ({},{}) and dimension ({},{})",
		origin_x, origin_y, side_x, side_x
	);
	for x in (origin_x - side_x)..(origin_x + side_x) {
		for y in (origin_y - side_y)..(origin_y + side_y) {
			set_pixel(pixels, (dim_x, dim_y), (x, y), (255, 0, 0));
		}
	}
}
fn write_floats(v: &Vec<f32>, f: &mut std::fs::File) -> std::io::Result<()> {
	for float in v {
		f.write_f32::<LittleEndian>(*float)?;
	}
	Ok(())
}
fn add_buildings(
	objects: &Vec<Cube>,
	pixels: &mut Vec<u8>,
	image_width: usize,
	image_height: usize,
) {
	for ob in objects.iter() {
		if {
			match ob.material {
				Material::Light(_) => false,
				_ => true,
			}
		} && ob.origin.y() > 1.0
		{
			draw_rectangle(
				pixels,
				image_width,
				image_height,
				ob.dim_x as usize,
				ob.dim_z as usize,
				ob.origin.x() as i64 as usize,
				ob.origin.z() as i64 as usize,
			);
		}
	}
}

const HOMOGENIZATION_FALLOFF: f32 = 3.0;

fn homogenize_signals(signals: &mut Vec<f32>, dim_x: usize) {
	//This method finds places where there is no signal, and sets it to the value of a nearby point where there is.

	//We cannot simply set a part of the signal map to another part in a parallel fashion.
	//This may locally lead to some slight weirdness, but that's the price we pay.
	let copied_signals = signals.clone();
	signals.par_iter_mut().enumerate().for_each(|(i, s)| {
		if *s - -140.0 < std::f32::EPSILON {
			//unwrapping: let coord = x + y * scene.width;
			let y = i / dim_x;
			let x = i % dim_x;

			//We go out to 5 spaces away
			'circles: for j in 1..=2 {
				if x < j || x > dim_x - j || y < j || y > dim_x - j {
					//Don't go past the edge. This makes the edges a bit ragged.
					break;
				}
				//Select the lower left corner of the square around (x,y) at distance j
				let mut x_ = x - j;
				let mut y_ = y - j;
				//Go right
				for _ in 1..=(j + 1) {
					if copied_signals[x_ + y_ * dim_x] > -140.0 {
						*s = copied_signals[x_ + y_ * dim_x] - HOMOGENIZATION_FALLOFF * (j as f32);
						break 'circles;
					}
					x_ += 1;
				}
				//Go up
				for _ in 1..=(j + 1) {
					if copied_signals[x_ + y_ * dim_x] > -140.0 {
						*s = copied_signals[x_ + y_ * dim_x] - HOMOGENIZATION_FALLOFF * (j as f32);
						break 'circles;
					}
					y_ += 1;
				}
				//Go left on top
				for _ in 1..=(j + 1) {
					if copied_signals[x_ + y_ * dim_x] > -140.0 {
						*s = copied_signals[x_ + y_ * dim_x] - HOMOGENIZATION_FALLOFF * (j as f32);
						break 'circles;
					}
					x_ -= 1;
				}
				//Go down on the left
				for _ in 1..=(j + 1) {
					if copied_signals[x_ + y_ * dim_x] > -140.0 {
						*s = copied_signals[x_ + y_ * dim_x] - HOMOGENIZATION_FALLOFF * (j as f32);
						break 'circles;
					}
					y_ -= 1;
				}
			}
		}
	});
}

pub fn generate_sigmap(
	filename_base: &str,
	scene: &Config,
	do_times: bool,
	do_angles: bool,
	do_png: bool,
) {
	let image_width = scene.width;
	let image_height = scene.height;

	let stations = find_lights(&scene.objects);
	let start = Instant::now();
	stations.par_iter().for_each(|s| {
		//for s in stations.iter() {
		let (mut signals, times, angles) = generate_signal(&s, &scene, do_times, do_angles);
		homogenize_signals(&mut signals, image_width);
		let freq = match s.material {
			Material::Light(l) => Ok(l.frequency),
			_ => Err(""),
		}
		.expect("Station does not have light material");
		let filename =
			filename_base.to_string() + "_" + &(s.id as i32).to_string() + "_" + &freq.to_string();
		//let s = serde_json::to_string(&signals).expect("Failed to serialize");
		//std::fs::write(filename.clone() + ".json", &s).expect("Could not write to file");
		let mut signals_file =
			File::create(filename.clone() + ".data").expect("Failed to create data file");
		write_floats(&signals, &mut signals_file).expect("Could not write data");
		if do_times {
			let mut times_file =
				File::create(filename.clone() + ".times").expect("Failed to create times file");

			write_floats(&times, &mut times_file).expect("Could not write times");
		}
		if do_angles {
			let mut angles_file =
				File::create(filename.clone() + ".angles").expect("Failed to create angles file");

			write_floats(&angles, &mut angles_file).expect("Could not write angles");
		}
		if do_png {
			let mut pixels: Vec<u8> = vec![0; (image_width + 1) * (image_height + 1) * 3];
			signal_to_pixels(&signals, &mut pixels, image_width, image_height);
			add_buildings(&scene.objects, &mut pixels, image_width, image_height);
			write_image(&(filename + ".png"), &pixels, (image_width, image_height))
				.expect("error writing image");
		}
	});
	println!("Frame time: {}ms", start.elapsed().as_millis());
}
