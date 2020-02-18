#!/usr/bin/env sh
GENESIS_PATH=${1:-../genesis.txn}
if [ ! -f "$GENESIS_PATH" ]; then
    echo "Genesis file not found"
    exit 1
fi
GENESIS_PATH=$(realpath "$GENESIS_PATH")

DOCKER_BUILDKIT=1 docker build -t indy-vdr-test-python \
    -f Dockerfile.test-python .. || exit 1

docker run --rm -ti \
	-v "$GENESIS_PATH:/home/indy/config/genesis.txn" \
	indy-vdr-test-python
