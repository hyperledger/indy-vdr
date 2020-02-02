#!/usr/bin/env sh
GENESIS_PATH=${1:-genesis.txn}
cd ..
if [ ! -f "$GENESIS_PATH" ]; then
    echo "Genesis file not found, please place in root directory ($GENESIS_PATH)"
    exit 1
fi
DOCKER_BUILDKIT=1 docker build -t indy-vdr-test-python \
    --build-arg "GENESIS_PATH=$GENESIS_PATH" \
    -f docker/Dockerfile.test-python . || exit 1
docker run --rm -ti indy-vdr-test-python
