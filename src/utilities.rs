// IMAGE SCREEN FUNCTIONS

use std::f64::consts::PI;
use druid::{EventCtx, Point};
use image::{DynamicImage, GenericImage, load_from_memory_with_format};
use screenshots::Screen;
use crate::{Annotation, GrabData};
use crate::constants::{BORDER_WIDTH, BUTTON_HEIGHT, NORMAL_BIG_IMAGE_LIMIT, OFFSET_X, OFFSET_Y, SMALL_IMAGE_LIMIT};

pub fn compute_offsets(ctx: &mut EventCtx, data: &mut GrabData) {
    // Calculate the offset to center mouse positions in the Image
    //let size = ctx.size();
    let widget_size = ctx.window().get_size();
    let image_width = data.image_size.0;
    let image_height = data.image_size.1;
    let x_offset = (widget_size.width - image_width) / 2.0;
    // take into account BUTTON_HEIGHT * 3.0 more height to make highliter and text widget visible
    let y_offset = (widget_size.height - BUTTON_HEIGHT * 3.0 - image_height) / 2.0;

    if !data.first_screen {
        if x_offset < 1.0 {
            data.offsets.0 = x_offset;
            data.offsets.1 = y_offset - OFFSET_Y;
        } else {
            data.offsets.0 = x_offset - OFFSET_X;
            data.offsets.1 = y_offset - OFFSET_Y;
        }
    }
}

pub fn load_image(data: &GrabData ) -> DynamicImage {
    let image;

    if data.image_data_new.is_empty() {
        image = load_from_memory_with_format(&data.image_data_old, image::ImageFormat::Png)
            .expect("Failed to load image from memory");
    } else {
        image = load_from_memory_with_format(&data.image_data_new, image::ImageFormat::Png)
            .expect("Failed to load image from memory");
    }

    image
}

pub fn image_to_buffer(image: DynamicImage) -> Vec<u8> {
    let mut png_buffer = std::io::Cursor::new(Vec::new());
    image.write_to(&mut png_buffer, image::ImageFormat::Png)
        .expect("Failed to Save Modified Image");

    png_buffer.into_inner()
}

pub fn make_rectangle_from_points(data: &GrabData ) -> Option<(f64,f64,f64,f64)> {
    if data.positions.is_empty() {
        return None;
    }
    let (mut min_x,mut max_y) = (0.0,0.0);
    let (mut max_x,mut min_y) = (0.0,0.0);
    let (p1x,p1y) = data.positions[0];
    let (p2x,p2y) = data.positions[data.positions.len() - 1];

    if p1x < p2x && p1y < p2y {
        // p1 smaller than p2
        min_x = p1x;
        min_y = p1y;
        max_x = p2x;
        max_y = p2y;
    } else if p1x > p2x && p1y > p2y {
        // p2 smaller than p1
        min_x = p2x;
        min_y = p2y;
        max_x = p1x;
        max_y = p1y;
    } else if p1x < p2x && p1y > p2y {
        // partenza in basso a sx
        min_x = p1x;
        min_y = p2y;
        max_x = p2x;
        max_y = p1y;
    } else if p1x > p2x && p1y < p2y {
        // partenza in alto a dx
        min_x = p2x;
        min_y = p1y;
        max_x = p1x;
        max_y = p2y;
    }

    Some((min_x,min_y,max_x,max_y))
}

pub fn compute_circle_center_radius(data: &GrabData, min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> (f64,f64) {
    // compute the center
    let center_x = ((max_x as f64 - data.offsets.0) - (min_x as f64 - data.offsets.0)) / 2.0 +  (min_x as f64 - data.offsets.0);
    let center_y = ((max_y as f64 - data.offsets.1) - (min_y as f64 - data.offsets.1)) / 2.0 +  (min_y as f64 - data.offsets.1);
    (center_x ,center_y)
}

pub fn compute_arrow_points(data: &GrabData) -> Option<((Point,Point),(Point,Point),(Point,Point))> {
    if data.positions.is_empty() {
        return None;
    }
    let main_line_p0 = Point::new(data.positions[0].0 - data.offsets.0, data.positions[0].1 - data.offsets.1);
    let main_line_p1 = Point::new(data.positions[data.positions.len()-1].0 - data.offsets.0, data.positions[data.positions.len()-1].1 - data.offsets.1);

    //direzione = endX - startX , endY - startY
    let direction = Point::new((data.positions[data.positions.len() - 1].0 - data.offsets.0) - (data.positions[0].0 - data.offsets.0),
                               (data.positions[data.positions.len() - 1].1 - data.offsets.1) - (data.positions[0].1 - data.offsets.1));
    //lunghezza = ipotenusa teorema di pitagora
    let arrow_length = ((direction.x.powi(2) + direction.y.powi(2)) as f64).sqrt();
    // angolo tra asseX e freccia
    let angle = (direction.y as f64).atan2(direction.x as f64);
    // lunghezza punta della freccia [settata ad un terzo]
    let arrow_tip = arrow_length/3.0;

    // Calcola punti della punta della freccia
    let arrow_x1 = (data.positions[data.positions.len() - 1].0 - data.offsets.0) - (direction.x / arrow_length);
    let arrow_y1 = (data.positions[data.positions.len() - 1].1 - data.offsets.1) - (direction.y / arrow_length);

    let arrow_l0_p0 = main_line_p1;
    let arrow_l0_p1 = Point::new(arrow_x1 - arrow_tip * (angle + PI / 6.0).cos(),arrow_y1 - arrow_tip * (angle + PI / 6.0).sin());
    let arrow_l1_p0 = main_line_p1;
    let arrow_l1_p1 = Point::new(arrow_x1 - arrow_tip * (angle - PI / 6.0).cos(),arrow_y1 - arrow_tip * (angle - PI / 6.0).sin());

    // main line point couple, first line point couple, second line point couple
    Some(((main_line_p0,main_line_p1),(arrow_l0_p0, arrow_l0_p1),(arrow_l1_p0, arrow_l1_p1)))
}

pub fn compute_highlighter_points(data: &GrabData) -> Option<(Point, Point, Point, Point)> {
    if data.positions.is_empty() {
        return None;
    }
    // draw line with first and last position, then clear the vector
    let point1 = Point::new(data.positions[0].0 - data.offsets.0, data.positions[0].1 - data.offsets.1);
    let point2 = Point::new(data.positions[data.positions.len()-1].0 - data.offsets.0,data.positions[data.positions.len()-1].1 - data.offsets.1);

    // Define your margin and the two points representing the line segment
    let highlighter_width = data.highlighter_width;

    // Calculate the slope of the line
    let dx = point2.x - point1.x;
    let dy = point2.y - point1.y;
    let slope = dy / dx;

    // Calculate the angle of the line with respect to the horizontal axis
    let angle = slope.atan();

    // Calculate the change in x and y coordinates for the margin
    let delta_x = highlighter_width * (angle + PI / 2.0).cos();
    let delta_y = highlighter_width * (angle + PI / 2.0).sin();

    // Create the four vertices of the rectangle
    let rect_point1 = Point::new(point1.x + delta_x, point1.y + delta_y);
    let rect_point2 = Point::new(point1.x - delta_x, point1.y - delta_y);
    let rect_point3 = Point::new(point2.x - delta_x, point2.y - delta_y);
    let rect_point4 = Point::new(point2.x + delta_x, point2.y + delta_y);

    Some((rect_point1,rect_point2,rect_point3,rect_point4))
}

// Image Resizing
pub fn resize_image(image: DynamicImage, data: &mut GrabData) -> (f64, f64) {
    let screen = Screen::all().unwrap()[0];
    let scale_factor_x ;
    let scale_factor_y;

    if image.width() >= (screen.display_info.width as f64 * NORMAL_BIG_IMAGE_LIMIT) as u32 || image.height() >= (screen.display_info.height as f64 * NORMAL_BIG_IMAGE_LIMIT) as u32 {
        // NORMAL OR BIG IMAGE (>= 50% of the screen)
        scale_factor_x = image.width() as f64 / (screen.display_info.width as f64 * 1.6);
        scale_factor_y = image.height() as f64 / (screen.display_info.height as f64 * 1.6);

    } else if image.width() <= (screen.display_info.width as f64 * SMALL_IMAGE_LIMIT) as u32 && image.height() <= (screen.display_info.height as f64 * SMALL_IMAGE_LIMIT) as u32 {
        // VERY SMALL IMAGE (<= 20% of the screen)
        scale_factor_x = 0.25;
        scale_factor_y = 0.25;
        //image = image.resize((screen.display_info.width / 4), (screen.display_info.height / 4), FilterType::Nearest);
    }else{
        // SMALL IMAGE (20% of the screen < size < 50% of the screen)
        scale_factor_x = (image.width() as f64 * 1.4) / (screen.display_info.width as f64);
        scale_factor_y = (image.height() as f64 * 1.4) / (screen.display_info.height as f64);
    }

    let aspect_ratio = image.width() as f64 / image.height() as f64;
    let desired_width = (screen.display_info.width as f64) * scale_factor_x;
    let desired_height = (screen.display_info.height as f64 - 7.0 * BUTTON_HEIGHT) * scale_factor_y;
    // Calculate the scaled dimensions while preserving aspect ratio
    let (mut scaled_width, mut scaled_height) = if image.width() as f64 / desired_width > image.height() as f64 / desired_height {
        // Fit by width
        (desired_width, desired_width / aspect_ratio as f64)
    } else {
        // Fit by height
        (desired_height * aspect_ratio as f64, desired_height)
    };

    //let window_size = Size::new( scaled_width,(scaled_height + BUTTON_HEIGHT * 7.0));

    if image.width() as f64>0.9* screen.display_info.width as f64 || image.height() as f64 > 0.9*screen.display_info.height as f64 {
        // if window size becames bigger than the monitor, rescale
        let big_factor = (image.width()as f64/screen.display_info.width as f64).max(image.height() as f64/screen.display_info.height as f64)+0.1;
        scaled_width /= big_factor;
        scaled_height /= big_factor;
    }

    data.image_size = (scaled_width,scaled_height);
    // assign scale factors to data
    // data.scale_factors.push(((image.width() as f64 / scaled_width),(image.height() as f64 / scaled_height)));
    data.scale_factors.0 = image.width() as f64 / scaled_width;
    data.scale_factors.1 = image.height() as f64 / scaled_height;

    (scaled_width,scaled_height)
}

// Reset Data Function
pub fn reset_data(data: &mut GrabData) {
    // set data fields to their initial state
    data.image_data_old = vec![];
    data.image_data_new = vec![];
    data.press = false;
    data.first_screen = true;
    data.scale_factors = (1.0,1.0);
    data.positions = vec![];
    data.offsets = (0.0,0.0);
    data.hotkey_new = vec![];
    data.hotkey_pressed = vec![];
    data.set_hot_key = false;
    data.input_hotkey_error = (false,"Invalid Input: Wrong Hotkey.".to_string());
    data.trigger_ui = false;
    data.annotation = Annotation::None;
    data.text_annotation = "".to_string();
}

struct ScreenImage {
    screen: screenshots::Screen,
    image: screenshots::Image,
}

pub fn compute_screening_coordinates(_data: &mut GrabData) -> (i32,i32,i32,i32) {
    /* get monitors info
    let mut accumulator = (0,0);
    for screen in Screen::all().unwrap() {
        accumulator.0 += screen.display_info.x as i32;
        accumulator.1 += screen.display_info.y as i32;
        data.monitors_info.push(accumulator);
    }*/

    // Capture all screens
    let screen_images = Screen::all()
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>();

    // Compute coordinates of combined image
    let x_min = screen_images.iter().map(|s| s.display_info.x).min().unwrap();
    let y_min = screen_images.iter().map(|s| s.display_info.y).min().unwrap();
    let x_max = screen_images
        .iter()
        .map(|s| s.display_info.x + s.display_info.width as i32)
        .max()
        .unwrap();
    let y_max = screen_images
        .iter()
        .map(|s| s.display_info.y + s.display_info.height as i32)
        .max()
        .unwrap();

    (x_min,y_min,x_max,y_max)
}

pub fn screen_all(min_x_grab: i32,min_y_grab: i32,max_x_grab: i32,max_y_grab: i32,data: &mut GrabData) {
    // Capture all screens
    let screen_images = Screen::all()
        .unwrap()
        .into_iter()
        .map(|screen| {
            let image = screen.capture().unwrap();
            ScreenImage { screen, image }
        })
        .collect::<Vec<_>>();

    let (x_min,y_min,x_max,y_max) = compute_screening_coordinates(data);
    let offset = (x_min, y_min);
    let size = ((x_max - x_min) as u32, (y_max - y_min) as u32);

    let mut img = DynamicImage::new_rgba8(size.0,size.1);
    for screen_image in screen_images {
        let screenshot = image::io::Reader::new(std::io::Cursor::new(screen_image.image.to_png(None).unwrap()))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();
        img.copy_from(
            &screenshot,
            (screen_image.screen.display_info.x - offset.0) as u32,
            (screen_image.screen.display_info.y - offset.1) as u32,
        )
            .unwrap();
    }

    img = img.crop(
        (((min_x_grab as f64 - data.offsets.0) * data.scale_factors.0) + BORDER_WIDTH) as u32,
        (((min_y_grab as f64 - data.offsets.1) * data.scale_factors.1) + BORDER_WIDTH) as u32,
        (((max_x_grab as f64- data.offsets.0) - ((min_x_grab as f64 - data.offsets.0) + 2.0 * BORDER_WIDTH)) * data.scale_factors.0) as u32,
        (((max_y_grab as f64- data.offsets.1) - ((min_y_grab as f64 - data.offsets.1) + 2.0 * BORDER_WIDTH)) * data.scale_factors.1) as u32
    );

    data.image_data_old = image_to_buffer(img);
}