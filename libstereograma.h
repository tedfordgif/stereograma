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
} // extern "C"
