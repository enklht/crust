#!/usr/bin/env fish

# Check args
if test (count $argv) -lt 1 -o (count $argv) -gt 2
    printf "Usage: my_cc.fish [--lex|--parse|--codegen] /path/to/source.c\n" >&2
    exit 2
end

if test (count $argv) -eq 2
    set flag $argv[1]
    set input $argv[2]
else
    set input $argv[1]
end

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

# Step 2: Compile -> assembly
./target/debug/my_cc -o "$asm" $flag "$pre"
if test $status -ne 0
    printf "Error: compilation to assembly failed\n" >&2
    exit 5
end

# We can remove the preprocessed file now
rm -f -- "$pre"

# Step 3: Assemble & link
if test -z "$flag"
    gcc "$asm" -o "$out"
    if test $status -ne 0
        # avoid leaving partial executable
        rm -f -- "$out"
        printf "Error: assembly/linking failed\n" >&2
        exit 6
    end
end

# If we reach here, success. Remove assembly file and exit 0.
rm -f -- "$asm"
exit 0
