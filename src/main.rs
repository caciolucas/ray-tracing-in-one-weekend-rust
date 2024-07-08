mod camera;
mod hit;
mod material;
mod ray;
mod sphere;
mod vec;

use std::io::{Read, Write, BufWriter};
use std::fs::File;
use std::sync::Arc;

use roxmltree::Document;

use rand::prelude::*;
use rayon::prelude::*;

use camera::Camera;
use hit::{Hit, World};
use material::{Dielectric, Lambertian, Metal};
use ray::Ray;
use sphere::Sphere;
use vec::{Color, Point3, Vec3};

use crate::material::Scatter;

fn ray_color(r: &Ray, world: &World, depth: u64) -> Color {
    if depth <= 0 {
        // If we've exceeded the ray bounce limit, no more light is gathered
        return Color::new(0.0, 0.0, 0.0);
    }

    if let Some(rec) = world.hit(r, 0.001, f64::INFINITY) {
        if let Some((attenuation, scattered)) = rec.mat.scatter(r, &rec) {
            attenuation * ray_color(&scattered, world, depth - 1)
        } else {
            Color::new(0.0, 0.0, 0.0)
        }
    } else {
        let unit_direction = r.direction().normalized();
        let t = 0.5 * (unit_direction.y() + 1.0);
        (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
    }
}

fn value_parser(values: &str) -> (f64, f64, f64) {
    let parts: Vec<&str> = values.split_whitespace().collect();

    // Parse each part into an f64 variable
    (
        parts[0].parse::<f64>().expect("Failed to parse number 1"),
        parts[1].parse::<f64>().expect("Failed to parse number 2"),
        parts[2].parse::<f64>().expect("Failed to parse number 3"),
    )
}

fn xml_parser(xml: &str) -> (String, World, Camera) {
    let doc = Document::parse(xml).expect("Failed to parse XML");

    let mut img_name = String::new();

    // Camera infos
    let mut lookfrom = Point3::new(0.0, 0.0, 0.0);
    let mut lookat = Point3::new(0.0, 0.0, 0.0);
    let mut vup = Vec3::new(0.0, 0.0, 0.0);
    let vfov = 20.0;
    let aspect_ratio = 3.0 / 2.0;
    let mut aperture = 0.0;
    let dist_to_focus = 10.0;

    // World infos
    let mut world = World::new();
    let ground_mat = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    let ground_sphere = Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_mat);

    world.push(Box::new(ground_sphere));

    // Last material added
    let mut last_mat : Arc<dyn Scatter> = Arc::new(Lambertian::new(Color::new(0.0, 0.0, 0.0)));

    // Traversing XML tree
    for node in doc.descendants() {
        if node.is_element() {
            let tag_name = node.tag_name().name();

            match tag_name {
                "film" => {
                    if let Some(value) = node.attribute("filename") {
                        img_name = value.to_string();
                    } else {
                        println!("Missing output file name in XML, used default.ppm");
                        img_name = "default.ppm".to_string();
                    }
                },
                "camera" => {
                    // Parsing look-from
                    if let Some(attr) = node.attribute("look_from") {
                        let value = value_parser(attr);
                        lookfrom = Point3::new(value.0, value.1, value.2);
                    } else {
                        panic!("Missing camera look from position!");
                    }

                    // Parsing look-at
                    if let Some(attr) = node.attribute("look_at") {
                        let value = value_parser(attr);
                        lookat = Point3::new(value.0, value.1, value.2);
                    } else {
                        panic!("Missing camera look at position!");
                    }

                    // Parsing up 
                    if let Some(attr) = node.attribute("up") {
                        let value = value_parser(attr);
                        vup = Point3::new(value.0, value.1, value.2);
                    } else {
                        panic!("Missing camera up position!");
                    }

                    // Parsing aperture 
                    if let Some(attr) = node.attribute("aperture") {
                        aperture = attr.parse()
                            .expect("Failed to parse camera aperture.");
                    } else {
                        panic!("Missing camera aperture!");
                    }

                },
                "material" => {
                    let mut mat_type = String::new();
                    let mut color = Color::new(0.0, 0.0, 0.0);

                    // Parsing material type 
                    if let Some(attr) = node.attribute("type") {
                        mat_type = attr.to_string();
                    } else {
                        panic!("Missing material type!");
                    }

                    // Parsing material color 
                    if let Some(attr) = node.attribute("color") {
                        let value = value_parser(attr);
                        color = Color::new(value.0, value.1, value.2);
                    } else { }

                    match mat_type.as_str() {
                        "lambertian" => last_mat = Arc::new(Lambertian::new(color)),
                        "metal" => {
                            let mut fuzz = 0.0;

                            // Parsing fuzziness 
                            if let Some(attr) = node.attribute("fuzz") {
                                fuzz = attr.parse()
                                    .expect("Failed to parse material fuzziness.");
                            } else {
                                panic!("Missing material fuzziness.");
                            }

                            last_mat = Arc::new(Metal::new(color, fuzz));
                        },
                        "dielectric" => {
                            let mut refrect = 0.0;

                            // Parsing fuzziness 
                            if let Some(attr) = node.attribute("refrect_idx") {
                                refrect = attr.parse()
                                    .expect("Failed to parse material refrective index.");
                            } else {
                                panic!("Missing material refrective index.");
                            }

                            last_mat = Arc::new(Dielectric::new(refrect));
                        },
                        _ => panic!("The material doesn't exists!."),
                    }
                },
                "object" => {
                    let mut center = Point3::new(0.0, 0.0, 0.0);
                    let mut rad = 0.0;
                    
                    // Parsing object center 
                    if let Some(attr) = node.attribute("center") {
                        let value = value_parser(attr);
                        center = Point3::new(value.0, value.1, value.2);
                    } else {
                        panic!("Missing object center!");
                    }

                    // Parsing object radius 
                    if let Some(attr) = node.attribute("radius") {
                        rad = attr.parse()
                            .expect("Failed to parse object radius.");
                    } else {
                        panic!("Missing object radius.");
                    }

                    // Adding sphere to the world
                    let new_obj = Sphere::new(center, rad, last_mat.clone());
                    world.push(Box::new(new_obj));

                },
                _ => { },
            }
        } else if node.is_text() { }
    }

    let cam = Camera::new(
        lookfrom,
        lookat,
        vup,
        vfov,
        aspect_ratio, 
        aperture,
        dist_to_focus,
    );

    (img_name, world, cam)
}

fn main() {
    // Reading XML scene 
    let mut xml_name = String::new();
    print!("Please enter the name of the XML scene file: ");
    std::io::stdout().flush().unwrap();

    std::io::stdin()
        .read_line(&mut xml_name)
        .expect("Failed to read line");

    let mut xml_file = File::open(xml_name.trim()).expect("Unable to open file.");
    let mut xml_contents = String::new();
    xml_file.read_to_string(&mut xml_contents).expect("Unable to read file.");

    // Parsing XML contents
    let (img_name, world, cam) = xml_parser(&xml_contents);
 
    // Image
    const ASPECT_RATIO: f64 = 3.0 / 2.0;
    const IMAGE_WIDTH: u64 = 1200;
    const IMAGE_HEIGHT: u64 = ((IMAGE_WIDTH as f64) / ASPECT_RATIO) as u64;
    const SAMPLES_PER_PIXEL: u64 = 500;
    const MAX_DEPTH: u64 = 50;

    let new_file = File::create(&img_name)
        .expect("Failed to create file.");
    let mut new_file = BufWriter::new(new_file);
    
    writeln!(new_file, "P3").expect("Filed to write");
    writeln!(new_file, "{} {}", IMAGE_WIDTH, IMAGE_HEIGHT).expect("Filed to write");
    writeln!(new_file, "255").expect("Filed to write");

    for j in (0..IMAGE_HEIGHT).rev() {
        eprintln!("Scanlines remaining: {}", j + 1);

        let scanline: Vec<Color> = (0..IMAGE_WIDTH)
            .into_par_iter()
            .map(|i| {
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                for _ in 0..SAMPLES_PER_PIXEL {
                    let mut rng = rand::thread_rng();
                    let random_u: f64 = rng.gen();
                    let random_v: f64 = rng.gen();

                    let u = ((i as f64) + random_u) / ((IMAGE_WIDTH - 1) as f64);
                    let v = ((j as f64) + random_v) / ((IMAGE_HEIGHT - 1) as f64);

                    let r = cam.get_ray(u, v);
                    pixel_color += ray_color(&r, &world, MAX_DEPTH);
                }

                pixel_color
            })
            .collect();

        for pixel_color in scanline {
            writeln!(new_file, "{}", pixel_color.format_color(SAMPLES_PER_PIXEL)).expect("Filed to write");
        }
    }

    eprintln!("Done.");

}
