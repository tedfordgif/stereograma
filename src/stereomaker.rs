extern crate libc;
use std::slice;
use std::convert::TryInto;
//use std::ptr;

#[no_mangle]
pub extern "C" fn composeDepthGeneric(
    depth: &mut u8, depth_width: usize, depth_height: usize,
    compose: &u8, compose_width: usize, compose_height: usize
)
{
    let depth = unsafe { slice::from_raw_parts_mut(depth, (depth_width * depth_height) as usize) };
    let compose = unsafe { slice::from_raw_parts(compose, (compose_width * compose_height) as usize) };

    compose_depth(depth, depth_width, compose, compose_width);
}

pub fn compose_depth(
    depth: &mut [u8], depth_width: usize,
    compose: &[u8], compose_width: usize
)
{
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

    #[test]
    fn test_compose_depth() {
        let mut d = [
            0, 1, 2, 3,
            2, 4, 5, 0,
            0, 5, 4, 0,
            0, 0, 0, 8
        ];
        let c = [
            1, 2,
            3, 0
        ];
        compose_depth(&mut d[..], 4, &c, 2);
        assert!(d == [
            1, 2, 2, 3,
            3, 4, 5, 0,
            1, 5, 4, 2,
            3, 0, 3, 8
        ], "{:?}", d);
        let mut d = [
            0, 1, 2, 3, 0,
            2, 4, 5, 0, 0,
            0, 5, 4, 0, 0,
            0, 0, 0, 8, 0,
            0, 0, 0, 8, 0
        ];
        compose_depth(&mut d[..], 5, &c, 2);
        assert!(d == [
            1, 2, 2, 3, 1,
            3, 4, 5, 0, 3,
            1, 5, 4, 2, 1,
            3, 0, 3, 8, 3,
            1, 2, 1, 8, 1
        ], "{:?}", d);
    }
}
