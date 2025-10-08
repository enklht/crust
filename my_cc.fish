#!/usr/bin/env fish
# fish-compiler-driver
# Usage: ./fish-compiler-driver /path/to/program.c
# Produces: /path/to/program (executable) on success

# Check args
if test (count $argv) -ne 1
    printf "Usage: %s /path/to/source.c\n" (basename $argv0) >&2
    exit 2
end

set input $argv[1]

if not test -f $input -a -r $input
    printf "Error: input file '%s' not found or not readable\n" $input >&2
    exit 3
end

# Derive paths
set dir (path dirname -- $input)
set name (path basename -E -- $input)
set pre "$dir/$name.i"
set asm "$dir/$name.s"
set out "$dir/$name"

# Cleanup function (always remove intermediate files)
function __cleanup --on-event EXIT
    # Remove intermediates if they exist
    test -n "$pre" && rm -f -- $pre
    test -n "$asm" && rm -f -- $asm
end

# Step 1: Preprocess
gcc -E -P "$input" -o "$pre"
if test $status -ne 0
    printf "Error: preprocessing failed\n" >&2
    exit 4
end

# Step 2: Compile -> assembly (STUB)
# Since the real compiler isn't written yet, use gcc -S as a stub to produce .s
gcc -S -O -fno-asynchronous-unwind-tables -fcf-protection=none "$pre" -o "$asm"
if test $status -ne 0
    printf "Error: compilation (stub) to assembly failed\n" >&2
    exit 5
end

# We can remove the preprocessed file now (spec requires deleting it)
rm -f -- "$pre"

# Step 3: Assemble & link
gcc "$asm" -o "$out"
if test $status -ne 0
    # avoid leaving partial executable
    rm -f -- "$out"
    printf "Error: assembly/linking failed\n" >&2
    exit 6
end

# If we reach here, success. Remove assembly file and exit 0.
rm -f -- "$asm"
exit 0
