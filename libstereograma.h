#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

extern "C" {

//bool composeDepthGeneric(uchar* depth_ptr, usize_t depth_width, usize_t depth_height,
//                         uchar* compose_ptr, usize_t compose_width, usize_t compose_height);
//void scaleLine(uint8_t *big, const uint8_t *original, int sizeoriginal);
//void scaleLine(uchar* big,const uchar *original,int sizeoriginal);
void composeDepthGeneric(uint8_t *depth,
                         uintptr_t depth_width,
                         uintptr_t depth_height,
                         const uint8_t *compose,
                         uintptr_t compose_width,
                         uintptr_t compose_height);

void scaleLine(uint8_t *big, const uint8_t *original, uintptr_t sizeoriginal);
void renderFromMap(
    const uint8_t *map,
    uint32_t map_width,
    uint32_t map_height,
    uint32_t max_depth,
    uint32_t min_depth,
    const uint8_t *pattern,
    uint32_t pattern_width,
    uint32_t pattern_height,
    uint8_t *result,
    uint32_t dpi,
    uint32_t observer_distance,
    uint32_t eye_separation
    );

} // extern "C"
