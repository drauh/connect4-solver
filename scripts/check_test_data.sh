#!/usr/bin/env bash
set -euo pipefail

DATASET=${1:-test-data/middle-medium}
BIN_CMD=${BIN_CMD:-"cargo run --release --bin score --quiet"}

if [ ! -f "$DATASET" ]; then
  echo "Dataset file '$DATASET' not found" >&2
  exit 1
fi

# Stream predictions through a FIFO so progress prints as we process cases.
predicted_pipe=$(mktemp -u)
mkfifo "$predicted_pipe"
trap 'rm -f "$predicted_pipe"' EXIT

cut -d' ' -f1 "$DATASET" | eval "$BIN_CMD" > "$predicted_pipe" &
predict_pid=$!

total=0
matches=0
mismatches=0

exec 3<"$predicted_pipe"
while read -r position expected _; do
  if ! read -r predicted_line <&3; then
    echo "Prediction output ended early at case $total" >&2
    kill "$predict_pid" 2>/dev/null || true
    exit 1
  fi
  predicted_score=$(echo "$predicted_line" | awk '{print $2}')
  total=$((total + 1))
  if [ "$expected" = "$predicted_score" ]; then
    matches=$((matches + 1))
    # echo "Case $total: match (expected=$expected, got=$predicted_score)"
  else
    mismatches=$((mismatches + 1))
    echo "Mismatch #$total position $position expected $expected got $predicted_score" >&2
  fi
done < "$DATASET"
wait "$predict_pid"

echo "Checked $total positions from $DATASET"
echo "Matches: $matches"
echo "Mismatches: $mismatches"

exit $((mismatches != 0))
