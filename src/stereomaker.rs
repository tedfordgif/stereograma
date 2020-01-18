extern crate libc;
use arr_macro::arr;
use image::imageops::resize;
use image::FilterType::Triangle;
use image::{ImageBuffer, Rgba, RgbaImage};
use std::convert::TryInto;
use std::slice;
//use std::ptr;

/*#[no_mangle]
pub extern "C" fn renderFromMap() -> &[u8]
{

}*/

pub fn render(
    map: &[u8],
    map_width: u32,
    map_height: u32,
    max_depth: u32,
    min_depth: u32,
    pattern: &RgbaImage,
    dpi: u32,
    observer_distance: u32,
    eye_separation: u32,
) -> RgbaImage {
    assert!(map.len() == (map_width * map_height) as usize);

    // Oversampling is 6, but against a line scaled to double width.
    let oversam = 6;
    let vwidth: u32 = oversam * map_width * 2;

    let mut look_left = vec![0u32; vwidth as usize];
    let mut look_right = vec![0u32; vwidth as usize];

    let y_shift = dpi / 16;
    let v_eye_sep = eye_separation * oversam * 2;

    // Use similar triangles to calculate max/min separation of matching pixels on virtual screen.
    let vmax_sep = (oversam * eye_separation * max_depth * 2) / (max_depth + observer_distance);
    let vmin_sep = (2 * oversam * eye_separation * min_depth) / (min_depth + observer_distance);
    // Pattern must be at least this wide, as must the source image.
    let max_sep = vmax_sep / (oversam * 2);
    let pattern_width = pattern.width();
    assert!(pattern_width > max_sep);
    assert!(map_width > max_sep);

    let max_height = max_depth - min_depth;

    // Scale pattern image to repeat horizontally at separation distance without oversampling
    // and vertically without oversampling and factor of 2.
    let pattern = resize(
        pattern,
        vmax_sep / oversam + 1,
        (pattern.height() * (max_sep + 1)) / pattern.width(),
        Triangle,
    );
    let pattern_height = pattern.height();
    // Create lookup table for pattern rows.
    let pattern_rows: Vec<Vec<&Rgba<u8>>> = pattern.rows().map(|r| r.collect()).collect();

    let mut result: RgbaImage = ImageBuffer::new(map_width, map_height);
    let mut v_curr_result_line: RgbaImage = ImageBuffer::new(vwidth, 1);

    // s is start
    // Starts in the middle of the image, horizontally, minus half the vmax_sep.
    let s = vwidth / 2 - vmax_sep / 2;
    // Middle of pattern.
    let pattern_offset = vmax_sep - (s % vmax_sep);

    let mut sep = 0u32;

    // TODO: Create lookup table for virtual separation distance by depth.
    let depthsep: [u32; 256] = {
        let mut depth_index = 0;
        arr![
            {
                let feature_z = max_depth-depth_index*max_height/256;
                depth_index += 1;
                ((v_eye_sep*feature_z)/(feature_z+observer_distance))
            };
            256]
    };

    let mut doubled_map_line = vec![0u8; (map_width * 2) as usize];

    for ((y, result_row), map_row) in result
        .enumerate_rows_mut()
        .zip(map.chunks_exact(map_width as usize))
    {
        // TODO: initialize look[LR] to 0.
        for x in 0..vwidth {
            look_left[x as usize] = x;
            look_right[x as usize] = x;
        }

        // Stretch depth map line to double width.
        // This is where the factor of two comes from!
        // It's a tradeoff between having a larger map and running this function
        // on smaller chunks (single row at a time), I assume. Doing it this way
        // will also *always* create smooth, averaged edges, whereas a larger map
        // could still have large jumps. This function should maybe be called
        // "scaleAndSmoothLine".
        scale_line(&mut doubled_map_line, map_row);

        // Link look_left and look_right arrays
        // TODO: Can we shorten some iterations due to oversampling?
        // Maybe, might have something to do with ratio of oversampling to sep
        // x is virtual index, (column and oversampling)
        for x in (vmin_sep / 2)..(vmax_sep / 2) {
            if x % oversam == 0 {
                // Reset sep to start oversampling again.
                sep = depthsep[doubled_map_line[(x / oversam) as usize] as usize];
            }

            let left_u32 = (x as i32) - (sep as i32) / 2;
            let right = (left_u32 + sep as i32) as usize;
            if left_u32 >= 0 {
                let left = left_u32 as usize;
                let mut visual = true;
                // The look_left and look_right if clauses aren't independent,
                // especially because of oversampling.
                if (look_left[right] as usize) != right {
                    // Right pt already linked.
                    if (look_left[right] as usize) < left {
                        // Deeper than current, so break old links.
                        look_right[look_left[right] as usize] = look_left[right];
                        look_left[right] = right as u32;
                    } else {
                        visual = false;
                    }
                }
                if (look_right[left] as usize) != left {
                    if (look_right[left] as usize) > right {
                        look_left[look_right[left] as usize] = look_right[left];
                        look_right[left] = left as u32;
                    } else {
                        visual = false;
                    }
                }
                if visual {
                    // Link both sides.
                    look_left[right] = left as u32;
                    look_right[left] = right as u32;
                }
            }
        }

        for x in (vmax_sep / 2)..(vwidth - vmax_sep / 2) {
            if x % oversam == 0 {
                // Reset sep to start oversampling again.
                sep = depthsep[doubled_map_line[(x / oversam) as usize] as usize];
            }

            let left = (x - sep / 2) as usize;
            let right = left + sep as usize;
            if right < vwidth as usize {
                let mut visual = true;
                // The look_left and look_right if clauses aren't independent,
                // especially because of oversampling.
                if (look_left[right] as usize) != right {
                    // Right pt already linked.
                    if (look_left[right] as usize) < left {
                        // Deeper than current, so break old links.
                        look_right[look_left[right] as usize] = look_left[right];
                        look_left[right] = right as u32;
                    } else {
                        visual = false;
                    }
                }
                if (look_right[left] as usize) != left {
                    if (look_right[left] as usize) > right {
                        look_left[look_right[left] as usize] = look_right[left];
                        look_right[left] = left as u32;
                    } else {
                        visual = false;
                    }
                }
                if visual {
                    // Link both sides.
                    look_left[right] = left as u32;
                    look_right[left] = right as u32;
                }
            }
        }

        // Fill first vmin_sep pixels with pattern, starting with s.
        // IDEA: Use iterators instead of put_pixel
        for x in s..(s + vmin_sep) {
            // Get color from pattern.
            v_curr_result_line.put_pixel(
                x,
                0,
                *pattern_rows[((y + ((x - s) / vmax_sep) * y_shift + pattern_height)
                    % pattern_height) as usize]
                    [(((x + pattern_offset) % vmax_sep) / oversam) as usize],
            );
        }

        // Fill center (s+vmin_sep to s+vmax_sep) of line.
        let mut lastlinked: u32 = 0; // dummy initial value
        for x in (s + vmin_sep)..(s + vmax_sep) {
            if (look_left[x as usize] == x) || (look_left[x as usize] < s) {
                // Not linked or linked to something in the left side of the image.
                if lastlinked == (x - 1) {
                    // Use adjacent color to reduce "twinkling" (retinal rivalry).
                    v_curr_result_line.put_pixel(x, 0, *v_curr_result_line.get_pixel(x - 1, 0));
                } else {
                    // Get "new" color from previous row (y_shift).
                    v_curr_result_line.put_pixel(
                        x,
                        0,
                        *pattern_rows[((y + ((x - s) / vmax_sep) * y_shift + pattern_height)
                            % pattern_height) as usize]
                            [(((x + pattern_offset) % vmax_sep) / oversam) as usize],
                    );
                }
            } else {
                // Linked to a value we know, so use that.
                v_curr_result_line.put_pixel(x, 0, *v_curr_result_line.get_pixel(look_left[x as usize], 0));
                // Keep track of the last pixel to be constrained.
                lastlinked = x;
            }
        }

        // Fill right half of line.
        for x in (s + vmax_sep)..vwidth {
            if look_left[x as usize] == x {
                if lastlinked == (x - 1) {
                    v_curr_result_line.put_pixel(x, 0, *v_curr_result_line.get_pixel(x - 1, 0));
                } else {
                    v_curr_result_line.put_pixel(
                        x,
                        0,
                        *pattern_rows[((y + ((x - s) / vmax_sep) * y_shift + pattern_height)
                            % pattern_height) as usize]
                            [(((x + pattern_offset) % vmax_sep) / oversam) as usize],
                    );
                }
            } else {
                v_curr_result_line.put_pixel(x, 0, *v_curr_result_line.get_pixel(look_left[x as usize], 0));
                lastlinked = x;
            }
        }

        // Fill left half of line.
        // Opposite of right side, except we can use pixels from right side.
        lastlinked = vwidth;
        for x in (0..s).rev() {
            if look_right[x as usize] == x {
                if lastlinked == (x + 1) {
                    v_curr_result_line.put_pixel(x, 0, *v_curr_result_line.get_pixel(x + 1, 0));
                } else {
                    v_curr_result_line.put_pixel(
                        x,
                        0,
                        *pattern_rows[((y + ((s - x) / vmax_sep + 1) * y_shift + pattern_height)
                            % pattern_height) as usize]
                            [(((x + pattern_offset) % vmax_sep) / oversam) as usize],
                    );
                }
            } else {
                v_curr_result_line.put_pixel(x, 0, *v_curr_result_line.get_pixel(look_right[x as usize], 0));
                lastlinked = x;
            }
        }

        // Downsample to original width.
        let curr_result_scaled_line = resize(&v_curr_result_line, map_width, 1, Triangle);

        // Copy line into result.
        for ((_, _, result_p), cur_p) in result_row.zip(curr_result_scaled_line.pixels()) {
            *result_p = *cur_p;
        }
    }
    result
}

#[no_mangle]
pub extern "C" fn composeDepthGeneric(
    depth: &mut u8,
    depth_width: usize,
    depth_height: usize,
    compose: &u8,
    compose_width: usize,
    compose_height: usize,
) {
    let depth = unsafe { slice::from_raw_parts_mut(depth, (depth_width * depth_height) as usize) };
    let compose =
        unsafe { slice::from_raw_parts(compose, (compose_width * compose_height) as usize) };

    compose_depth(depth, depth_width, compose, compose_width);
}

#[inline]
pub fn compose_depth(depth: &mut [u8], depth_width: usize, compose: &[u8], compose_width: usize) {
    let depth_rows = depth.chunks_exact_mut(depth_width);
    let compose_rows = compose.chunks_exact(compose_width).cycle();
    let zipped_rows = depth_rows.zip(compose_rows);

    for (depth_row, compose_row) in zipped_rows {
        for (depth_pixel, compose_pixel) in depth_row.iter_mut().zip(compose_row.iter().cycle()) {
            if compose_pixel > depth_pixel {
                *depth_pixel = *compose_pixel;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn scaleLine(big: *mut u8, original: *const u8, sizeoriginal: usize) {
    let safe_big =
        unsafe { slice::from_raw_parts_mut(big, (sizeoriginal * 2).try_into().unwrap()) };
    let safe_original =
        unsafe { slice::from_raw_parts(original, sizeoriginal.try_into().unwrap()) };
    scale_line(safe_big, safe_original);
}

#[inline]
fn scale_line(big: &mut [u8], original: &[u8]) {
    // approach: loop
    assert!(original.len() > 1);
    assert!(original.len() * 2 == big.len());
    big[0] = original[0];
    for i in 0..original.len() - 1 {
        big[i * 2 + 1] = ((3 * original[i] as u16 + original[i + 1] as u16) / 4) as u8;
        big[i * 2 + 2] = ((original[i] as u16 + 3 * original[i + 1] as u16) / 4) as u8;
    }
    big[original.len() * 2 - 1] = original[original.len() - 1];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_line() {
        let mut b = vec![0; 32];
        let o = vec![
            10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160,
        ];
        scale_line(&mut b, &o);
        assert!(
            b == [
                10, 12, 17, 22, 27, 32, 37, 42, 47, 52, 57, 62, 67, 72, 77, 82, 87, 92, 97, 102,
                107, 112, 117, 122, 127, 132, 137, 142, 147, 152, 157, 160
            ],
            "{:?}",
            b
        );
    }

    #[test]
    fn test_compose_depth() {
        let mut d = [0, 1, 2, 3, 2, 4, 5, 0, 0, 5, 4, 0, 0, 0, 0, 8];
        let c = [1, 2, 3, 0];
        compose_depth(&mut d[..], 4, &c, 2);
        assert!(
            d == [1, 2, 2, 3, 3, 4, 5, 0, 1, 5, 4, 2, 3, 0, 3, 8],
            "{:?}",
            d
        );
        let mut d = [
            0, 1, 2, 3, 0, 2, 4, 5, 0, 0, 0, 5, 4, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 8, 0,
        ];
        compose_depth(&mut d[..], 5, &c, 2);
        assert!(
            d == [1, 2, 2, 3, 1, 3, 4, 5, 0, 3, 1, 5, 4, 2, 1, 3, 0, 3, 8, 3, 1, 2, 1, 8, 1],
            "{:?}",
            d
        );
    }
}
