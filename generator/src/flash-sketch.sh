#!/usr/bin/sh

echo "[TERMINAL] COMPILING SKETCH"

arduino-cli compile $HOME/projects/pusit/generator/ --fqbn arduino:avr:nano:cpu=atmega328old

echo "[TERMINAL] SKETCH COMPILED"

echo "[TERMINAL] UPLOADING SKETCH"


arduino-cli upload $HOME/projects/pusit/generator/ --port /dev/ttyUSB0 --fqbn arduino:avr:nano:cpu=atmega328old 

echo "[TERMINAL] SKETCH FLASHED"
