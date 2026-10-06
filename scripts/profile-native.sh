#!/bin/sh
# Developer smoke recordings; the installed app needs neither this script nor a shell.
# Build first. Run with an otherwise idle machine; screenshots follow the sample window.
set -eu
cosmogon_root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
cosmogon_binary="$cosmogon_root/target/release/cosmogon"
cosmogon_output=${1:-$(mktemp -d "${TMPDIR:-/tmp}/cosmogon-profile.XXXXXX")}
mkdir -p "$cosmogon_output"

if [ ! -x "$cosmogon_binary" ]; then
    echo "Build the native release application before running this script." >&2
    exit 1
fi

record_scene() {
    cosmogon_scene=$1
    shift
    echo "Recording ${cosmogon_scene}..."
    "$cosmogon_binary" --new lab --seed 1 --speed 0 "$@" \
        --profile "$cosmogon_output/$cosmogon_scene.json" \
        --profile-warmup 120 --profile-frames 600 \
        --capture "$cosmogon_output/$cosmogon_scene.png" --capture-metadata --capture-after 760 --quit \
        > "$cosmogon_output/$cosmogon_scene.log" 2>&1
    test -s "$cosmogon_output/$cosmogon_scene.json"
    test -s "$cosmogon_output/$cosmogon_scene.png"
}

record_scene saturn --focus Saturn --distance 5 --panel performance
record_scene mars-surface --focus Mars --distance 1.003 --latlon 18.4,noon --hide-ui
record_scene galaxy --focus Sun --distance 2e12 --hide-ui

echo "Reports, captures and logs: $cosmogon_output"
