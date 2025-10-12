test *flags:
  cargo build
  ./writing-a-c-compiler-tests/test_compiler ./crust.fish {{flags}}
