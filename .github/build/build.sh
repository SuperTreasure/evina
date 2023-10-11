#! /usr/bin/env bash

set -eEuo pipefail

changelog() {
    changes="${INPUT_CHANGES}\n\n| 文件 | SHA-256 |\n| :---: | :---: |"
    files=$(ls artifac)
    for file in $files
    do 
    changes="${changes}\n| $file | $(shasum -a 256 artifac/$file | awk '{print $1}') |"
    done
    echo -e $changes > changes.md
}


build () {
    mkdir -p build
    if [[ "${INPUT_TARGET}" =~ "windows" ]]; then
    cross build --bin ${INPUT_BIN} --release --target=${INPUT_TARGET}
    upx --best --ultra-brute $PWD/target/${INPUT_TARGET}/release/evina.exe || true
    zip -j build/evina-${GITHUB_REF#refs/tags/}-${INPUT_TARGET}.zip $PWD/target/${INPUT_TARGET}/release/evina.exe
    elif [[ "${INPUT_TARGET}" =~ "linux" ]]; then
    cross build --bin ${INPUT_BIN} --release --target=${INPUT_TARGET}
    upx --best --ultra-brute $PWD/target/${INPUT_TARGET}/release/evina || true
    tar -cvf build/evina-${GITHUB_REF#refs/tags/}-${INPUT_TARGET}.tar.xz -C $PWD/target/${INPUT_TARGET}/release evina
    fi
}

"$@"
