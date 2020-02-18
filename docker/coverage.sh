#!/usr/bin/env sh

COVERAGE_CONTAINER=${COVERAGE_CONTAINER:-indy-vdr-coverage}

cd ..

DOCKER_BUILDKIT=1 docker build --progress=plain -t $COVERAGE_CONTAINER \
    -f docker/Dockerfile.coverage . || exit 1

if [ ! -d coverage ]; then
	mkdir coverage
fi

docker run --rm -ti \
	-e "WRAPPER_DEBUG=1" \
	--mount type=bind,source="/$(pwd)"/libindy_vdr,target=/volume/libindy_vdr,readonly \
	--mount type=bind,source="/$(pwd)"/indy-vdr-proxy,target=/volume/indy-vdr-proxy,readonly \
	--mount type=bind,source="/$(pwd)"/coverage,target=/volume/coverage \
	--mount type=volume,source=$COVERAGE_CONTAINER-target,destination=/volume/target,consistency=delegated \
	--mount type=volume,source=$COVERAGE_CONTAINER-vendor,destination=/volume/vendor,consistency=delegated \
	$COVERAGE_CONTAINER
