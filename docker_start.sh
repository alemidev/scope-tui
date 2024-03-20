#!/bin/sh

docker build -t scope_tui:latest . &&
	docker run --rm -it --name scope_tui -e "PULSE_SERVER=unix:${XDG_RUNTIME_DIR}/pulse/native" -v "${XDG_RUNTIME_DIR}/pulse/native:${XDG_RUNTIME_DIR}/pulse/native" -v "$HOME/.config/pulse/cookie:/.config/pulse/cookie" scope_tui
