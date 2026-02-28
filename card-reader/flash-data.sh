#!/usr/bin/sh

# Big thanks to the author of this blog:
# https://www.instructables.com/How-to-Upload-Files-to-ESP32-LittleFS-File-System/

echo "[TERMINAL] CREATING BIN FILE"

mklittlefs -c $HOME/projects/pusit/terminal/data/ -s 0x160000 -p 0x100 $HOME/projects/pusit/terminal/data.bin

echo "[TERMINAL] CREATED BIN FILE"

echo "[TERMINAL] FLASHING BIN FILE"

esptool --chip esp32 --port /dev/ttyUSB0 write-flash 0x290000 $HOME/projects/pusit/terminal/data.bin

echo "[TERMINAL] FLASHED BIN FILE"
