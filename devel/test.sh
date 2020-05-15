#! /usr/bin/bash

SRC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"
PARAMS=$@

podman run --network=host --rm -it -v $SRC_DIR:/rpick:z -e RPICK_CONFIG="/rpick/devel/config.yml" \
	-e RUST_BACKTRACE=1 rpick:dev \
	bash -c "cd /rpick && cargo test && cargo clippy --all-targets --all-features -- -D warnings && cargo doc"
