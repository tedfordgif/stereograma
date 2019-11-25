extern crate libc;
use std::slice;
use std::convert::TryInto;
use image::imageops::resize;
use image::ImageBuffer;
use image::FilterType::Triangle;
//use std::ptr;

#[no_mangle]
pub extern "C" fn renderFromMap() -> GenericImage
{

}

pub fn render(map: &mut [u8], max_depth: u32, min_depth: u32,
              pattern: &ImageBuffer,
              dpi: u32, observer_distance: u32, eye_separation: u32) -> GenericImage
{
    // Oversampling is 6, but against a line scaled to double width.
    let oversam = 6u8;
    let vwidth : usize = oversam*map_width*2;

    let map_width = map.width();
    let map_height = map.height();

    let mut lookL = vec![0usize; vwidth];
    let mut lookR = vec![0usize; vwidth];

    let yShift = dpi/16;
    let veyeSep=eye_separation*oversam*2;

    // Use similar triangles to calculate max/min separation of matching pixels on virtual screen.
    let vmax_sep=((oversam*eye_separation*max_depth*2)/(max_depth+observer_distance));
    let vmin_sep=((2*oversam*eye_separation*min_depth)/(min_depth+observer_distance));
    // Pattern must be at least this wide
    let max_sep=vmax_sep/(oversam*2);
    let pattern_width = pattern.width();
    assert!(pattern_width > max_sep);

    let max_height = max_depth - min_depth;

    // Scale pattern image to repeat horizontally at separation distance without oversampling
    // and vertically without oversampling and factor of 2.
    let pattern=resize(pattern, vmax_sep/oversam+1,(pattern.height()*(max_sep+1))/pattern.width(), Triangle);
    // TODO: assert!(pattern.depth == 32);
    let pattern_height=pattern.height();
    let mut result : RgbaImage = ImageBuffer::new(map_width, map_height);
    let mut vCurResultLine : RgbaImage = ImageBuffer::new(vwidth, 1);

    // s is start
    // Starts in the middle of the image, horizontally, minus half the vmax_sep.
    let s=vwidth/2-vmax_sep/2;
    // Middle of pattern.
    let pattern_offset=vmax_sep-(s % vmax_sep);

    let sep=0u32;

    // Create lookup table for pattern rows.
    let rows = pattern.rows().map(|r| { r.collect() }).collect();

    // TODO: Create lookup table for virtual separation distance by depth.
    let depthsep = (0..256).map(|depth_index| {
        let featureZ = max_depth-depth_index*max_height/256;
        ((veyeSep*featureZ)/(featureZ+observer_distance))
    }).collect();

    uchar * mapptr=(uchar *)malloc(map.width()*2);
    for (y=0;y<map_height;y++) {
        // TODO: initialize look[LR] to 0.
        for (x=0; x<vwidth; x++) {
            lookL[x]=x;
            lookR[x]=x;
        }

        // Stretch depth map line to double width.
        // This is where the factor of two comes from!
        // It's a tradeoff between having a larger map and running this function
        // on smaller chunks (single row at a time), I assume. Doing it this way
        // will also *always* create smooth, averaged edges, whereas a larger map
        // could still have large jumps. This function should maybe be called
        // "scaleAndSmoothLine".
        scaleLine(mapptr,map.scanLine(y),map_width);

        // Link lookL and lookR arrays
        // TODO: Can we shorten some iterations due to oversampling?
        // Maybe, might have something to do with ratio of oversampling to sep
        // x is virtual index, (column and oversampling)
        for (x=0; x<vwidth; x++) {
            if (x%oversam==0) {
                // Reset sep to start oversampling again.
                sep=depthsep[mapptr[x/oversam]];
            }

            left=x-sep/2; right=left+sep;
            if ((left>=0) && (right<vwidth)) {
                visual=true;
                // The lookL and lookR if clauses aren't independent, 
                // especially because of oversampling.
                if (lookL[right]!=right) {
                    // Right pt already linked.
                    if (lookL[right]<left) {
                        // Deeper than current, so break old links.
                        lookR[lookL[right]]=lookL[right];
                        lookL[right]=right;
                    }
                    else visual=false;
                }
                if (lookR[left]!=left) {
                    if (lookR[left]>right) {
                        lookL[lookR[left]]=lookR[left];
                        lookR[left]=left;
                    }
                    else visual=false;
                }
                if (visual) {
                    // Link both sides.
                    lookL[right]=left;
                    lookR[left]=right;
                }
            }
        }


        lineptr=(uint*)vCurResultLine.scanLine(0);

        // Fill first vmin_sep pixels with pattern, starting with s.
        // TODO: convert to std::copy-like in Rust.
        dataptr=lineptr+s;
        for (x=s; x<vwidth && x<s+vmin_sep; x++, dataptr++) {
            // Get color from pattern.
            *dataptr=patternptr[(y+((x-s)/vmax_sep)*yShift+pattern_height) % pattern_height][((x+pattern_offset) % vmax_sep)/oversam];
        }

        // Fill center (s+vmin_sep to s+vmax_sep) of line.
        lastlinked=-10; // dummy initial value
        for (; x<vwidth && x<s+vmax_sep; x++, dataptr++) {
            if ((lookL[x]==x) || (lookL[x]<s)) {
                // Not linked or linked to something in the left side of the image.
                if (lastlinked==(x-1))
                    // Use adjacent color to reduce "twinkling" (retinal rivalry).
                    *dataptr=*(dataptr-1);
                else {
                    // Get "new" color from previous row (yShift).
                    *dataptr=patternptr[(y+((x-s)/vmax_sep)*yShift+pattern_height) % pattern_height][((x+pattern_offset) % vmax_sep)/oversam];
                }
            }
            else {
                // Linked to a value we know, so use that.
                *dataptr=*(lineptr+lookL[x]);
                // Keep track of the last pixel to be constrained.
                lastlinked=x; 
            }
        }

        // Fill right half of line.
        for (; x<vwidth; x++, dataptr++) {
            if ((lookL[x]==x)) {
                if (lastlinked==(x-1))
                    *dataptr=*(dataptr-1);
                else {
                    *dataptr=patternptr[(y+((x-s)/vmax_sep)*yShift+pattern_height) % pattern_height][((x+pattern_offset) % vmax_sep)/oversam];
                }
            }
            else {
                *dataptr=*(lineptr+lookL[x]);
                lastlinked=x; 
            }
        }

        // Fill left half of line.
        // Opposite of right side, except we can use pixels from right side.
        dataptr=lineptr+s-1;
        lastlinked=-10;
        for (x=s-1; x>=0; x--, dataptr--) {
            if (lookR[x]==x) {
                if (lastlinked==(x+1))
                    *dataptr=*(dataptr+1);
                else {
                    *dataptr=patternptr[(y+((s-x)/vmax_sep+1)*yShift+pattern_height) % pattern_height][((x+pattern_offset) % vmax_sep)/oversam];
                }
            }
            else {
                *dataptr=*(lineptr+lookR[x]);
                lastlinked=x;
            }
        }

        // Downsample to original width.
        CurResultScaledLine = vCurResultLine.scaledToWidth(map_width,Qt::SmoothTransformation);
        // Copy line into result.
        memcpy(result.scanLine(y),CurResultScaledLine.scanLine(0),result.bytesPerLine());
    }

}

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

#[inline]
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

#[inline]
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
