#! /usr/bin/bash

SRC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"

podman build --pull -t rpick:dev -v $SRC_DIR:/rpick:z --force-rm=true \
	-f devel/Dockerfile-build
