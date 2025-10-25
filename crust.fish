#!/usr/bin/env fish

# Parse arguments
argparse \
    c/compile-only \
    /lex \
    /parse \
    /validate \
    /tacky \
    /codegen \
    /emit \
    /help \
    /lm \
    -- $argv
or exit

if set -q _flag_help
    printf "Usage: crust.fish [OPTIONS] /path/to/source.c\n\n"
    printf "Options:\n"
    printf "  --lex                  Stop after lexing stage\n"
    printf "  --parse                Stop after parsing stage\n"
    printf "  --validate             Stop after validation stage\n"
    printf "  --tacky                Stop after tacky generation stage\n"
    printf "  --codegen              Stop after code generation stage\n"
    printf "  --emit                 Stop after code emission stage\n"
    printf "  -c, --compile-only     Compile only, do not link\n"
    printf "  --lm                   Link with math library (-lm)\n"
    printf "  -h, --help             Show this help message\n"
    exit 0
end

# Check input file and extract gcc arguments
set gcc_args
for arg in $argv
    if string match -r -- '.+\.c$' "$arg"
        if not set -q input
            set input "$arg"
        else
            printf "Error: more than one .c file specified\n" >&2
            exit 2
        end
    else
        set -a gcc_args "$arg"
    end
end

if not set -q input
    printf "Error: no .c input file specified\n" >&2
    exit 2
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

# Set math flag
if set -q _flag_lm
    set math_flag -lm
end

# Derive paths
set dir (path dirname -- $input)
set name (path basename -E -- $input)
set out "$dir/$name"

trap "rm -f $out.i $out.s" EXIT

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

# Step 3: Assemble & link
if test -z "$flag"
    if set -q _flag_compile_only
        gcc -c "$out.s" -o "$out.o" $math_flag
        if test $status -ne 0
            rm -f -- "$out.o"
            printf "Error: assembly failed\n" >&2
            exit 6
        end
    else
        gcc "$out.s" -o "$out" $math_flag
        if test $status -ne 0
            # avoid leaving partial executable
            rm -f -- "$out"
            printf "Error: assembly/linking failed\n" >&2
            exit 6
        end
    end
end

exit 0
