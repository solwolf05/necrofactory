clean:
    rm -rf necro
    rm -f necro.tar.zst

build-linux: clean
    mkdir -p necro
    cargo build --release
    cp target/release/necrofactory necro
    cp -r mods necro
    cp -r assets necro
    tar --zstd -cf necro.tar.zst necro

build-windows: clean
    mkdir -p necro
    cargo build --release --target x86_64-pc-windows-gnu
    cp target/x86_64-pc-windows-gnu/release/necrofactory.exe necro
    cp -r mods necro
    cp -r assets necro
    tar --zstd -cf necro.tar.zst necro

run-windows: build-windows
    wine necro/necrofactory.exe

run-linux: build-linux
    necro/necrofactory
