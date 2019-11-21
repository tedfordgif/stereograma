extern crate libc;
//use libc::{c_int};
use std::slice;
use std::convert::TryInto;
//use std::ptr;
use image::Luma;
use image::ImageBuffer;
//use image::flat::{FlatSamples, SampleLayout};

#[no_mangle]
pub extern "C" fn composeDepthGeneric(
    depth: &mut u8, depth_width: usize, depth_height: usize,
    compose: &u8, compose_width: usize, compose_height: usize
) -> bool
{
    let mut depth = unsafe { slice::from_raw_parts_mut(depth, (depth_width * depth_height) as usize) };
    let compose = unsafe { slice::from_raw_parts(compose, (compose_width * compose_height) as usize) };

    let mut depth = match ImageBuffer::from_raw(depth_width as u32, depth_height as u32, depth) {
        None => return false, // Invalid size.
        Some(buf) => buf,
    };

    let compose = match ImageBuffer::from_raw(compose_width as u32, compose_height as u32, compose) {
        None => return false, // Invalid size.
        Some(buf) => buf,
    };

    let mut depth_rows = depth.rows_mut();
    let compose_rows = compose.rows().cycle();
    let zipped_rows = depth_rows.zip(compose_rows);

    for (depth_row, compose_row) in zipped_rows {
        for (depth_val, compose_val) in depth_row.zip(compose_row.cycle()) {
            if *compose_val > *depth_val {
                *depth_val = *compose_val;
            }
        }
    }

    return true;
}

/*
#[no_mangle]
pub extern "C" fn composeDepthGenericFlatSamples(
    depth: *const mut u8, depth_len: usize,
    depth_layout: *const SampleLayout
    compose: *const u8, compose_len: usize,
    compose_layout: *const SampleLayout
)
    -> bool
{
    let mut depth_samples = unsafe { slice::from_raw_parts(depth, len) };
    let depth_layout = unsafe { ptr::read(layout) };
    let compose_samples = unsafe { slice::from_raw_parts(compose, len) };
    let compose_layout = unsafe { ptr::read(layout) };

    let mut depth_buffer = FlatSamples {
        depth_samples,
        depth_layout,
        color_hint: None,
    };

    let compose_buffer = FlatSamples {
        compose_samples,
        compose_layout,
        color_hint: None,
    };

    let mut depth_view = match depth_buffer.as_view::<Luma<u8>>() {
        Err(_) => return false, // Invalid layout.
        Ok(view) => view,
    };

    let compose_view = match compose_buffer.as_view::<Luma<u8>>() {
        Err(_) => return false, // Invalid layout.
        Ok(view) => view,
    };

    
}
*/

#[no_mangle]
pub extern "C" fn scaleLine(big: *mut u8, original: *const u8, sizeoriginal: usize)
{
    let safe_big = unsafe { slice::from_raw_parts_mut(big, (sizeoriginal * 2).try_into().unwrap()) };
    let safe_original = unsafe { slice::from_raw_parts(original, sizeoriginal.try_into().unwrap()) };
    scale_line(safe_big, safe_original);
}

fn scale_line(big: &mut [u8], original: &[u8])
{
    // approach: loop
    assert!(original.len() > 1);
    assert!(original.len() * 2 == big.len());
    big[0]=original[0];
    for i in 0..original.len()-1 {
        big[i*2 + 1] = ((3 * original[i] as u16 + original[i+1] as u16)/4) as u8;
        big[i*2 + 2] = ((original[i] as u16 + 3 * original[i+1] as u16)/4) as u8;
    }
    big[original.len()*2-1] = original[original.len() - 1];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_line() {
        let mut b = vec![0; 32];
        let o = vec![10,20,30,40,50,60,70,80,90,100,110,120,130,140,150,160];
        scale_line(&mut b, &o);
        assert!(b == [10, 12, 17, 22, 27, 32, 37, 42, 47, 52, 57, 62, 67, 72, 77, 82, 87, 92, 97, 102, 107, 112, 117, 122, 127, 132, 137, 142, 147, 152, 157, 160], "{:?}", b);
    }
}
