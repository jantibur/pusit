#!/usr/bin/sh

echo "[TERMINAL] COMPILING SKETCH"

arduino-cli compile $HOME/projects/pusit/reader/ --fqbn esp32:esp32:esp32wrover

echo "[TERMINAL] SKETCH COMPILED"

echo "[TERMINAL] UPLOADING SKETCH"

arduino-cli upload $HOME/projects/pusit/reader/ --port /dev/ttyUSB0 --fqbn esp32:esp32:esp32wrover

echo "[TERMINAL] SKETCH UPLOADED"

echo "[TERMINAL] SKETCH FLASHED"
