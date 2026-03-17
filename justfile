clean:
    rm -rf necro

build-linux:
    mkdir -p necro
    cargo build
    cp target/debug/necrofactory necro
    cp mods necro/mods
    cp assets necro/assets
    tar --zstd -cf necro.tar.zst necro

build-windows:
    mkdir -p necro
    cargo build --release --target x86_64-pc-windows-gnu
    cp target/x86_64-pc-windows-gnu/debug/necrofactory.exe necro
    cp -r mods necro
    cp -r assets necro
    tar --zstd -cf necro.tar.zst necro

run-windows: build-windows
    wine necro/necrofactory.exe
