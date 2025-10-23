#!/usr/bin/env fish

# Parse arguments
argparse \
    c/compile-only \
    l/lex \
    p/parse \
    v/validate \
    t/tacky \
    g/codegen \
    e/emit \
    h/help \
    -- $argv
or exit

if set -q _flag_help
    printf "Usage: crust.fish [OPTIONS] /path/to/source.c\n\n"
    printf "Options:\n"
    printf "  -c, --compile-only     Compile only, do not link\n"
    printf "  -l, --lex              Stop after lexing stage\n"
    printf "  -p, --parse            Stop after parsing stage\n"
    printf "  -v, --validate         Stop after validation stage\n"
    printf "  -t, --tacky            Stop after tacky generation stage\n"
    printf "  -g, --codegen          Stop after code generation stage\n"
    printf "  -e, --emit             Stop after code emission stage\n"
    printf "  -h, --help             Show this help message\n"
    exit 0
end

# Check input file
if test (count $argv) -ne 1
    printf "Error: exactly one input file must be specified\n" >&2
    exit 2
end

set input $argv[1]
if not test -f $input -a -r $input
    printf "Error: input file '%s' not found or not readable\n" $input >&2
    exit 3
end

# Set compilation stage flag
set flag
if set -q _flag_lex
    set flag --lex
else if set -q _flag_parse
    set flag --parse
else if set -q _flag_validate
    set flag --validate
else if set -q _flag_tacky
    set flag --tacky
else if set -q _flag_codegen
    set flag --codegen
else if set -q _flag_emit
    set flag --emit
end

# Set compile-only flag
set compile_only 0
if set -q _flag_compile_only
    set compile_only 1
end

# Derive paths
set dir (path dirname -- $input)
set name (path basename -E -- $input)
set out "$dir/$name"

# Step 1: Preprocess
gcc -E -P "$input" -o "$out.i"
if test $status -ne 0
    printf "Error: preprocessing failed\n" >&2
    exit 4
end

# Step 2: Compile -> assembly
./target/debug/crust -o "$out.s" $flag "$out.i"
if test $status -ne 0
    printf "Error: compilation to assembly failed\n" >&2
    exit 5
end

# We can remove the preprocessed file now
rm -f -- "$out.i"

# Step 3: Assemble & link
if test -z "$flag"
    if test $compile_only -eq 1
        gcc -c "$out.s" -o "$out.o"
        if test $status -ne 0
            rm -f -- "$out.o"
            printf "Error: assembly failed\n" >&2
            exit 6
        end
    else
        gcc "$out.s" -o "$out"
        if test $status -ne 0
            # avoid leaving partial executable
            rm -f -- "$out"
            printf "Error: assembly/linking failed\n" >&2
            exit 6
        end
    end
end

# If we reach here, success. Remove assembly file and exit 0.
rm -f -- "$out.s"
exit 0
