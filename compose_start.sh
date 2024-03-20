#!/bin/sh

docker compose up -d

docker attach scope_tui

docker compose down
