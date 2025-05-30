#!/usr/bin/env bash
cd ~/repos/oort3/
set -e

SOURCE_AI=~/repos/oort_ai/target/bundle_output.rs
SCENARIO_PATH=shared/simulator/src/scenario
SCENARIO_AI_PATH=shared/builtin_ai/src

SCENARIOS=$(rg "(\w+):.*" ~/repos/oort_ai/best.txt -r '$1')
TIMES=$(rg "\w+: (.*)" ~/repos/oort_ai/best.txt -r '$1')

exec 5>&1;
for i in $(seq 0 $(($(echo "$SCENARIOS" | wc -l) - 1))); do
    SCENARIO=$(echo "$SCENARIOS" | sed -n "$((i + 1))p")
    TIME=$(echo "$TIMES" | sed -n "$((i + 1))p")

    if [ -f $SCENARIO_PATH/$SCENARIO.rs ]
    then
        enemy=$SCENARIO_AI_PATH/tutorial/${SCENARIO}_enemy.rs
    else
        enemy=$SCENARIO_AI_PATH/empty.rs
    fi
    if [ ! -f $enemy ]
    then
        enemy="$SCENARIO_AI_PATH/tutorial/tutorial_acceleration_initial.rs" # empty.rs stopped working for some reason
    fi
    export RUST_LOG=error
    printf "\r\033[K$SCENARIO:"
    result=$(./target/debug/battle -j $SCENARIO $SOURCE_AI $enemy)
    NEW_TIME=$(echo "$result" | jq ".[0].average_time")
    NEW_TIME=$(printf "%.3f" $NEW_TIME)
    if (( $(echo "$NEW_TIME $TIME" | awk '{print ($1 != $2)}') )); then
        printf "\r\033[K$SCENARIO: Old: $TIME, New: $NEW_TIME\n"
    fi
    printf "\r\033[K"
done
