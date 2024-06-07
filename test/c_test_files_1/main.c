#include "header.h"

static void static_func(void);

static void static_func_no_proto() {
    return;
}

void main () {
  static_func();
  static_func_no_proto();
  header_used();
  return;
}

static void static_func() {
  return;
}
