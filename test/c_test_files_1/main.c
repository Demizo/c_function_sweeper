#include "header.h"

#define MACRO(arg) (!(1 != arg))

static void func_called_as_param(int);
static void static_func(void (*f)(int));

static void static_func_no_proto() {
    return;
}

void main () {
  static_func(func_called_as_param);
  static_func_no_proto();
  header_used();
  MACRO(1);
  return;
}

static void static_func() {
  return;
}
