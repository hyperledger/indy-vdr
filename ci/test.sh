#/bin/bash

readonly command=$(basename ${0})

help(){
    echo "Usage: $command <subcommand> [options]\n"
    echo "Subcommands:"
    echo "    up"
    echo "    test"
    echo "    down"
    echo "Default behavior without a subcommand is up, test, down"
}

up(){
  docker build -f ci/indy-pool.dockerfile -t test_pool --build-arg pool_ip=10.0.0.2 ci
  docker network create --subnet=10.0.0.0/8 indy-sdk-network
  docker run -d --name indy_pool -p 9701-9708:9701-9708 --net=indy-sdk-network test_pool
}

run_tests(){
  cargo test  --manifest-path libindy_vdr/Cargo.toml --features local_nodes_pool
}

down(){
  docker stop indy_pool
  docker rm indy_pool
  docker network rm  indy-sdk-network
}

subcommand="$1"
case $subcommand in
  "-h" | "--help")
    help
    ;;
  "up")
    up
    ;;
  "test" | "tests")
    run_tests
    ;;
  "down")
    down
    ;;
  *)
    up
    run_tests
    down
    ;;
esac

