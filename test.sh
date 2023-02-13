#!/bin/sh

Xephyr -br -ac -noreset -screen 1280x800 :1 &
DISPLAY=:1 cargo run
killall Xephyr
