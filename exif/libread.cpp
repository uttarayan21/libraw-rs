#include "libraw.h"

extern "C" {
int libraw_read_datastream(void *data, void *ptr, size_t size, size_t nmemb);
}

int libraw_read_datastream(void *data, void *ptr, size_t size, size_t nmemb) {
  return ((LibRaw_file_datastream *)data)->read(ptr, size, nmemb);
}
