#!/usr/bin/env bash
set -e

cd $HOME/repos/oort3/
if [ $# -eq 0 ]
then
    echo "Usage: test.sh [source_ai] <scenario_name> [-n]"
    exit 1
fi

if [[ $# -eq 1 || $2 == "-n" ]]
then
    SOURCE_AI=~/repos/oort_ai/target/bundle_output.rs
    SCENARIO_NAME=$1
    SCENARIO=tutorial_$1
else
    SOURCE_AI=~/repos/oort_ai/$1
    SCENARIO_NAME=$2
    SCENARIO=tutorial_$2
fi
SCENARIO_PATH=shared/simulator/src/scenario
SCENARIO_AI_PATH=shared/builtin_ai/src

if [ -f $SCENARIO_PATH/$SCENARIO.rs ]
then
    enemy=$SCENARIO_AI_PATH/tutorial/${SCENARIO}_enemy.rs
else
    SCENARIO=$SCENARIO_NAME
    enemy=$SCENARIO_AI_PATH/empty.rs
fi

exec 5>&1;
if [ ! -f $enemy ]
then
    enemy="$SCENARIO_AI_PATH/tutorial/tutorial_acceleration_initial.rs" # empty.rs stopped working for some reason
fi
echo "Scenario: $SCENARIO"
echo "Source AI: $SOURCE_AI"
echo "Enemy AI: $enemy"
export RUST_LOG=error
result=$(./target/debug/battle -j $SCENARIO $SOURCE_AI $enemy)
cd ~/repos/oort_ai/
avg=$(echo $result | jq ".[0].average_time")
avg=$(printf "%.3f" $avg)
times=$(echo $result | jq '.[0].times | max as $m | map(.*1000 | round /1000 | if . < 10 and $m > 10 then "0\(.|tostring)" else .|tostring end | (split(".") | .[1] | length) as $l | if $l == 0 then . + ".000" elif $l < 3 then . + "0" * (3 - $l) else . end) | join(", ")' -r)
echo "Times:        $times"
printf "Average time: %.3f\n" $avg
losses=$(echo $result | jq '.[0].losses | length')
draws=$(echo $result | jq '.[0].draws | length')
if [[ $losses -gt 0 || $draws -gt 0 ]]
then
    echo "Draws:       $draws"
    echo "Losses:      $losses"
    exit 1
fi
TIMES=~/repos/oort_ai/best.txt
best=$(rg "$SCENARIO: (\d+.\d+)" $TIMES -r '$1' || echo "")
if [ -z "$best" ]
then
    best="1000"
    echo "Best time:    n/a"
    printf "$SCENARIO: %.3f\n" $avg >> $TIMES
else
    echo "Best time:    $best"
fi
if (( $(echo "$avg $best" | awk '{print ($1 < $2)}') )); then
    echo "New best time: $avg"
    if [[ ${!#} == "-n" ]]
    then
        exit 0
    fi
    sed -i "s/$SCENARIO: $best/$SCENARIO: $avg/" $TIMES
    git add .
    git commit -m "New best $SCENARIO: $avg" &> /dev/null
    git tag "$SCENARIO_NAME-$avg"
    git push &> /dev/null
    git push --tags &> /dev/null
fi
