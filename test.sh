#!/bin/sh

Xephyr -br -ac -noreset -screen 1440x900 :1 &
DISPLAY=:1 cargo run
killall Xephyr
