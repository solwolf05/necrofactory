build_dir := "necro"

linux_binary_debug := "target/x86_64-unknown-linux-gnu/debug/necrofactory"
windows_binary_debug := "target/x86_64-pc-windows-gnu/debug/necrofactory.exe"

linux_binary_release := "target/x86_64-unknown-linux-gnu/release/necrofactory"
windows_binary_release := "target/x86_64-pc-windows-gnu/release/necrofactory.exe"

run:
    mangohud cargo run

build mode="release" target="linux": clean
    mkdir -p {{ build_dir }}

    if [ "{{ target }}" = "linux" ]; then \
        cargo build {{ if mode == "release" { "--release" } else { "" } }} --target x86_64-unknown-linux-gnu; \
        cp {{ if mode == "release" { linux_binary_release } else { linux_binary_debug } }} {{ build_dir }}/necrofactory; \
    elif [ "{{ target }}" = "windows" ]; then \
        cargo build {{ if mode == "release" { "--release" } else { "" } }} --target x86_64-pc-windows-gnu; \
        cp {{ if mode == "release" { windows_binary_release } else { windows_binary_debug } }} {{ build_dir }}/necrofactory.exe; \
    fi

    cp -r mods {{ build_dir }}
    cp -r assets {{ build_dir }}

    tar --zstd -cf necro.tar.zst {{ build_dir }}

clean:
    rm -rf {{ build_dir }}
    rm -f necro.tar.zst

run-windows:
    wine {{ build_dir }}/necrofactory.exe

run-linux:
    {{ build_dir }}/necrofactory
