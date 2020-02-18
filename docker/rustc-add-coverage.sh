#!/bin/bash -e
# We don't want to build every dependency with profiling options enabled
# It's slow, has nothing in common with other builds, and generates lots of unused artifacts
# Based on: https://jbp.io/2017/07/19/measuring-test-coverage-of-rust-programs

get_crate_name() {
  while [[ $# -gt 1 ]] ; do
    v=$1
    case $v in
      --crate-name)
        echo $2
        return
        ;;
    esac
    shift
  done
}

CRATE_NAME=$(get_crate_name "$@")

CRATE_PATTERN=""
for crate in $COVERAGE_CRATES; do
  CRATE_PATTERN="${CRATE_PATTERN:+$CRATE_PATTERN|}$crate"
done

if [ ! -z "$CRATE_PATTERN" ] && [[ $CRATE_NAME =~ $CRATE_PATTERN ]]; then
  EXTRA=$COVERAGE_OPTIONS
  PROFILE="+profile"
else
  PROFILE=""
fi
if [ ! -z "$WRAPPER_DEBUG" ]; then
	echo "rustc crate: $CRATE_NAME $PROFILE"
fi

exec "$@" $EXTRA
