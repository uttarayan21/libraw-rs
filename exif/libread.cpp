#define LIBRAW_USE_DEPRECATED_IOSTREAMS_DATASTREAM
#include "libraw.h"

extern "C" {
int libraw_read_file_datastream(void *data, void *ptr, size_t size,
                                size_t nmemb);
int libraw_read_bigfile_datastream(void *data, void *ptr, size_t size,
                                   size_t nmemb);
int libraw_read_buffer_datastream(void *data, void *ptr, size_t size,
                                  size_t nmemb);
}

int libraw_read_file_datastream(void *data, void *ptr, size_t size,
                                size_t nmemb) {
  return ((LibRaw_file_datastream *)data)->read(ptr, size, nmemb);
}

int libraw_read_bigfile_datastream(void *data, void *ptr, size_t size,
                                   size_t nmemb) {
  return ((LibRaw_bigfile_datastream *)data)->read(ptr, size, nmemb);
}

int libraw_read_buffer_datastream(void *data, void *ptr, size_t size,
                                  size_t nmemb) {
  return ((LibRaw_buffer_datastream *)data)->read(ptr, size, nmemb);
}
