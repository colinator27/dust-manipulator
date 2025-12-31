use sdl3::rect::Rect;

use crate::server::ScreenshotData;

pub struct ImagePoint {
    pub x: u32,
    pub y: u32
}

pub struct FourPixelConfig {
    pub pixel_match_color_1_1: u32,
    pub pixel_replace_color_1_1: fn(u32) -> u32,
    pub pixel_coord_1_1: ImagePoint,
    pub pixel_coord_1_1_size: u32,
    pub pixel_match_color_1_2: u32,
    pub pixel_replace_color_1_2: fn(u32) -> u32,
    pub pixel_coord_1_2: ImagePoint,
    pub pixel_coord_1_2_size: u32,
    pub pixel_match_color_2_1: u32,
    pub pixel_replace_color_2_1: fn(u32) -> u32,
    pub pixel_coord_2_1: ImagePoint,
    pub pixel_coord_2_1_size: u32,
    pub pixel_match_color_2_2: u32,
    pub pixel_replace_color_2_2: fn(u32) -> u32,
    pub pixel_coord_2_2: ImagePoint,
    pub pixel_coord_2_2_size: u32,
}

pub fn make_four_pixel(image: &Vec<u8>, output_image: &mut Vec<u8>, screenshot1_data: &ScreenshotData, screenshot2_data: &ScreenshotData, config: &FourPixelConfig) {
    // Grab colors from screenshots
    // (note that this is meant to simulate basic image manipulation, although it just grabs data directly)
    let pixel_color_1_1 = (config.pixel_replace_color_1_1)(screenshot1_data.get_brightest_pixel(config.pixel_coord_1_1.x, config.pixel_coord_1_1.y, config.pixel_coord_1_1_size));
    let pixel_color_1_2 = (config.pixel_replace_color_1_2)(screenshot1_data.get_brightest_pixel(config.pixel_coord_1_2.x, config.pixel_coord_1_2.y, config.pixel_coord_1_2_size));
    let pixel_color_2_1 = (config.pixel_replace_color_2_1)(screenshot2_data.get_brightest_pixel(config.pixel_coord_2_1.x, config.pixel_coord_2_1.y, config.pixel_coord_2_1_size));
    let pixel_color_2_2 = (config.pixel_replace_color_2_2)(screenshot2_data.get_brightest_pixel(config.pixel_coord_2_2.x, config.pixel_coord_2_2.y, config.pixel_coord_2_2_size));

    // Build new image, replacing config colors with the replacement pixel colors
    let mut pos = 0;
    while pos < image.len() {
        let mut curr_color = u32::from_ne_bytes(image[pos..pos+4].try_into().unwrap());
        if curr_color == config.pixel_match_color_1_1 {
            curr_color = pixel_color_1_1;
        } else if curr_color == config.pixel_match_color_1_2 {
            curr_color = pixel_color_1_2;
        } else if curr_color == config.pixel_match_color_2_1 {
            curr_color = pixel_color_2_1;
        } else if curr_color == config.pixel_match_color_2_2 {
            curr_color = pixel_color_2_2;
        }
        output_image.extend(u32::to_ne_bytes(curr_color));
        pos += 4;
    }
}

pub fn clear_unwanted_pixels_dust(output_image: &mut Vec<u8>, screenshot_data: &ScreenshotData, world_view: Rect, is_early_frame: bool) {
    // Build new image, replacing irrelevant colors with black, and then brightening the image to white otherwise
    let data = &screenshot_data.data;
    if is_early_frame {
        // Remove darker gray and green
        let mut y = world_view.y;
        while y < world_view.y + world_view.h {
            let mut pos: usize  = ((y as u32 * screenshot_data.stride) + (world_view.x as u32 * 4)) as usize;
            let end_pos: usize = pos + (world_view.w as usize * 4);
            while pos < end_pos {
                let mut curr_color = u32::from_ne_bytes(data[pos..pos+4].try_into().unwrap());
                if curr_color == 0xFF494949 || curr_color == 0xFF22B14C {
                    curr_color = 0xFF000000;
                } else if curr_color != 0xFF000000 {
                    curr_color = 0xFFFFFFFF;
                }
                output_image.extend(u32::to_ne_bytes(curr_color));
                pos += 4;
            }
            y += 1;
        }
    } else {
        // Remove green only
        let mut y = world_view.y;
        while y < world_view.y + world_view.h {
            let mut pos: usize  = ((y as u32 * screenshot_data.stride) + (world_view.x as u32 * 4)) as usize;
            let end_pos: usize = pos + (world_view.w as usize * 4);
            while pos < end_pos {
                let mut curr_color = u32::from_ne_bytes(data[pos..pos+4].try_into().unwrap());
                if curr_color == 0xFF22B14C {
                    curr_color = 0xFF000000;
                } else if curr_color != 0xFF000000 {
                    curr_color = 0xFFFFFFFF;
                }
                output_image.extend(u32::to_ne_bytes(curr_color));
                pos += 4;
            }
            y += 1;
        }
    }
}

pub fn clear_unwanted_pixels_snowballs(output_image: &mut Vec<u8>, screenshot_data: &ScreenshotData, world_view: Rect) {
    // Build new image, replacing irrelevant colors with black, and then brightening the image to white otherwise
    let data = &screenshot_data.data;
    let mut y = world_view.y;
    while y < world_view.y + world_view.h {
        let mut pos: usize  = ((y as u32 * screenshot_data.stride) + (world_view.x as u32 * 4)) as usize;
        let end_pos: usize = pos + (world_view.w as usize * 4);
        while pos < end_pos {
            let mut curr_color = u32::from_ne_bytes(data[pos..pos+4].try_into().unwrap());
            if curr_color != 0xFFFFFFFF {
                curr_color = 0;
            }
            output_image.extend(u32::to_ne_bytes(curr_color));
            pos += 4;
        }
        y += 1;
    }
}
