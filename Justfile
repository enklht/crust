test *flags:
  cargo build
  ./writing-a-c-compiler-tests/test_compiler ./crust.fish {{flags}}

run fname *flags:
  cargo build
  ./crust.fish {{flags}} ./writing-a-c-compiler-tests/tests/{{fname}}
