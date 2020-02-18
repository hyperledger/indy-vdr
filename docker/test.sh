COVERAGE_CRATES="$@"

CRATE_PATTERN=""
for crate in $COVERAGE_CRATES; do
  CRATE_PATTERN="${CRATE_PATTERN:+$CRATE_PATTERN|}$crate"
done

echo $CRATE_PATTERN

[[ "one" =~ $CRATE_PATTERN ]] && echo 1 || echo 0
