test *flags:
  cargo build
  ./writing-a-c-compiler-tests/test_compiler ./crust.fish {{flags}}

run fname *flags:
  cargo build
  ./target/debug/crust {{flags}} ./writing-a-c-compiler-tests/tests/{{fname}}
