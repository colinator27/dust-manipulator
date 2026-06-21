use sdl3::rect::Rect;

use crate::server::ScreenshotData;

pub fn copy_pixels(output_image: &mut Vec<u8>, screenshot_data: &ScreenshotData, world_view: Rect) {
    let data = &screenshot_data.data;
    let mut y = world_view.y;
    while y < world_view.y + world_view.h {
        let mut pos: usize  = ((y as u32 * screenshot_data.stride) + (world_view.x as u32 * 4)) as usize;
        let end_pos: usize = pos + (world_view.w as usize * 4);
        while pos < end_pos {
            let curr_color = u32::from_ne_bytes(data[pos..pos+4].try_into().unwrap());
            output_image.extend(u32::to_ne_bytes(curr_color));
            pos += 4;
        }
        y += 1;
    }
}

pub fn clear_unwanted_pixels_dust(output_image: &mut Vec<u8>, screenshot_data: &ScreenshotData, world_view: Rect, is_early_frame: bool, brighten: bool) {
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
                if curr_color == 0xFF494949 || curr_color == 0xFF4CB122 {
                    curr_color = 0xFF000000;
                } else if brighten && curr_color != 0xFF000000 {
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
                if curr_color == 0xFF4CB122 {
                    curr_color = 0xFF000000;
                } else if brighten && curr_color != 0xFF000000 {
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
