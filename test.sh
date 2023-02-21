#!/bin/sh

Xephyr +xinerama +extension RANDR -screen 800x600+0+0 -screen 800x600+800+0 -ac :1 &
DISPLAY=:1 cargo run
killall Xephyr
