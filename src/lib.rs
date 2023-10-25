use extism_pdk::*;
use image::{Pixel, RgbaImage};
use lazy_static::lazy_static;
use rand::Rng;
use std::sync::{Arc, Mutex};

const PLACE_PNG_BYTES: &[u8] = include_bytes!("../assets/place.png");

lazy_static! {
    static ref PLACE_IMAGE: Arc<Mutex<RgbaImage>> = Arc::new(Mutex::new(
        image::load_from_memory(PLACE_PNG_BYTES)
            .expect("Unable to read image")
            .to_rgba8()
    ));
    static ref CURRENT_COORD: Arc<Mutex<(i32, i32)>> = Arc::new(Mutex::new((0, 0)));
    static ref DIRECTION: Arc<Mutex<(i32, i32)>> = Arc::new(Mutex::new((1, 1)));
}

#[host_fn]
extern "ExtismHost" {
    fn matricks_debug(msg: &str);
    fn matricks_info(msg: &str);
    fn matricks_warn(msg: &str);
    fn matricks_error(msg: &str);
}

#[plugin_fn]
pub fn setup(_: ()) -> FnResult<()> {
    unsafe {
        matricks_info("Starting the r/place trick.").expect("Unable to send log!");
    }

    // Grab the width and the height of the matrix
    let width: usize = config::get("width").unwrap().parse().unwrap();
    let height: usize = config::get("height").unwrap().parse().unwrap();

    // Create the random number generator
    let mut rng = rand::thread_rng();

    // Get the r/place image
    let place_img = PLACE_IMAGE.lock().unwrap();

    // Confirm that the viewer isn't too large
    if place_img.width() <= width as u32 || place_img.height() <= height as u32 {
        unsafe {
            matricks_error("Image is too small for the matrix!").expect("Unable to send log!");
        }
        panic!();
    }

    // Choose a random starting coordinate for the viewing info
    let mut current_coord = CURRENT_COORD.lock().unwrap();
    current_coord.0 = rng.gen_range(0..place_img.width()) as i32;
    current_coord.1 = rng.gen_range(0..place_img.height()) as i32;
    unsafe {
        matricks_info(&*format!(
            "Starting viewer at ({},{})",
            current_coord.0, current_coord.1
        ))
        .expect("Unable to send log!");
    }

    Ok(())
}

#[plugin_fn]
pub fn update(_: ()) -> FnResult<Json<Option<Vec<Vec<[u8; 4]>>>>> {
    // Get the width and height of the matrix
    let width: usize = config::get("width").unwrap().parse().unwrap();
    let height: usize = config::get("height").unwrap().parse().unwrap();

    // Grab some variables
    let place_img = PLACE_IMAGE.lock().unwrap();
    let mut current_coord = CURRENT_COORD.lock().unwrap();
    let mut direction = DIRECTION.lock().unwrap();

    let mut next_state: Vec<Vec<[u8; 4]>> = vec![];
    for y in 0..height {
        next_state.push(vec![]);
        for x in 0..width {
            let channels = place_img
                .get_pixel(
                    current_coord.0.abs() as u32 + x as u32,
                    current_coord.1.abs() as u32 + y as u32,
                )
                .channels();
            next_state[y].push([channels[2], channels[1], channels[0], channels[3]]);
        }
    }

    // Change the direction if the viewing window hits a wall
    if ((current_coord.0 + width as i32 >= place_img.width() as i32) && direction.0 > 0)
        || (current_coord.0 <= 0 && direction.0 < 0)
    {
        direction.0 *= -1;
    }
    if ((current_coord.1 + height as i32 >= place_img.height() as i32) && direction.1 > 0)
        || (current_coord.1 <= 0 && direction.1 < 0)
    {
        direction.1 *= -1;
    }

    // Move the viewing window
    current_coord.0 += direction.0;
    current_coord.1 += direction.1;

    Ok(Json(Some(next_state)))
}
