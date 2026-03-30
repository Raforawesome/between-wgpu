#!/opt/homebrew/bin/fish

for shader in src/*.slang
    set output (string replace .slang .wgsl $shader)
    slangc $shader -o $output
end

if contains -- "--release" $argv
    cargo run --release
else
    cargo run
end
