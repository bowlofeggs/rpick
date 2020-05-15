#! /usr/bin/bash

SRC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"
PARAMS=$@

podman run --network=host --rm -it -v $SRC_DIR:/rpick:z -e RPICK_CONFIG="/rpick/devel/config.yml" rpick:dev \
	bash -c "cd /rpick && cargo $PARAMS"
