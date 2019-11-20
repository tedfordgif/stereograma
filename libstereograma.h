#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

extern "C" {

typedef struct {
    uchar channels;
    usize_t channel_stride;
    u32 width;
    usize_t width_stride;
    u32 height;
    usize_t height_stride;
} SampleLayout;

//void scaleLine(uint8_t *big, const uint8_t *original, int sizeoriginal);
void scaleLine(uchar* big,const uchar *original,int sizeoriginal);
bool composeDepthGeneric(uchar* depth_ptr, int depth_len, SampleLayout depth_layout,
                         uchar* compose_ptr, int compose_len, SampleLayout compose_layout);
} // extern "C"
