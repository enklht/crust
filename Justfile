test *flags:
  cargo build
  ./writing-a-c-compiler-tests/test_compiler ./my_cc.fish {{flags}}

test-release *flags:
  cargo build --release
  ./writing-a-c-compiler-tests/test_compiler ./my_cc.fish {{flags}}
