#!/usr/bin/sh

echo "[TERMINAL] COMPILING SKETCH"

arduino-cli compile $HOME/projects/pusit/card-generator/ --fqbn arduino:avr:nano:cpu=atmega328old

echo "[TERMINAL] SKETCH COMPILED"

echo "[TERMINAL] UPLOADING SKETCH"


arduino-cli upload $HOME/projects/pusit/card-generator/ --port /dev/ttyUSB0 --fqbn arduino:avr:nano:cpu=atmega328old 

echo "[TERMINAL] SKETCH FLASHED"
