#!/usr/bin/env bash

cd $HOME/repos/oort3/
if [ $# -eq 0 ]
then
    echo "Usage: test.sh [source_ai] <scenario_name>"
    exit 1
fi

if [ $# -eq 1 ]
then
    SOURCE_AI=~/repos/oort_ai/target/bundle_output.rs
    SCENARIAO_NAME=$1
    SCENARIAO=tutorial_$1
else
    SOURCE_AI=~/repos/oort_ai/$1
    SCENARIAO_NAME=$2
    SCENARIAO=tutorial_$2
fi
SCENARIAO_PATH=shared/simulator/src/scenario
SCENARIAO_AI_PATH=shared/builtin_ai/src

if [ -f $SCENARIAO_PATH/$SCENARIAO.rs ]
then
    enemy=$SCENARIAO_AI_PATH/tutorial/${SCENARIAO}_enemy.rs
else
    SCENARIAO=$SCENARIAO_NAME
    enemy=$SCENARIAO_AI_PATH/empty.rs
fi

exec 5>&1;
if [ ! -f $enemy ]
then
    echo "Enemy AI not found: $enemy"
    enemy="shared/builtin_ai/src/empty.rs"
fi
echo "Scenario: $SCENARIAO"
echo "Source AI: $SOURCE_AI"
echo "Enemy AI: $enemy"
export RUST_LOG=error
result=$(./target/debug/battle -j $SCENARIAO $SOURCE_AI $enemy)
cd ~/repos/oort_ai/
avg=$(echo $result | jq ".[0].average_time")
times=$(echo $result | jq '.[0].times | map(.*1000 | floor /1000 | if . < 10 then "0\(.|tostring)" else .|tostring end | (split(".") | .[1] | length) as $l | if $l < 3 then . + "0" * (3 - $l) else . end) | join(", ")' -r)
echo "Times: $times"
printf "Average time: %.3f\n" $avg
TIMES=~/repos/oort_ai/best.txt
best=$(rg "$SCENARIAO: (\d+.\d+)" $TIMES -r '$1')
if [ -z "$best" ]
then
    best="1000"
    printf "$SCENARIAO: %.3f\n" $avg >> $TIMES
else
    echo "Best time:    $best"
fi
if (( $(echo "$avg $best" | awk '{print ($1 < $2)}') )); then
    echo "New best time: $avg"
    sed -i "s/$SCENARIAO: $best/$SCENARIAO: $avg/" $TIMES
    git add .
    git commit -m "New best $SCENARIAO: $avg" &> /dev/null
    git tag "$SCENARIAO_NAME-$avg"
    git push &> /dev/null
    git push --tags &> /dev/null
fi
