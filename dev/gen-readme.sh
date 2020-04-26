#!/bin/sh
cargo readme -r devbox -t ../dev/README.tpl -o README.md
cargo readme -r devbox-test-args -t ../dev/README.tpl -o README.md
cargo readme -r devbox-build -t ../dev/README.tpl -o README.md