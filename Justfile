test chapter stage:
  cargo build
  ./writing-a-c-compiler-tests/test_compiler ./crust.fish --latest-only --chapter {{chapter}} --stage {{stage}}

test-all *flags:
  cargo build
  ./writing-a-c-compiler-tests/test_compiler ./crust.fish {{flags}}

run fname *flags:
  cargo build
  ./crust.fish {{flags}} {{fname}}

check:
  RUSTFLAGS=-Awarnings cargo check
