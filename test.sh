#!/usr/bin/env bash

cd $HOME/repos/oort3/
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

exec 5>&1;

result=$(./target/debug/battle $SCENARIAO $SOURCE_AI shared/builtin_ai/src/tutorial/${SCENARIAO}_enemy.rs | tee /dev/fd/5)
cd ~/repos/oort_ai/
avg=$(echo $result | rg -o "Average time: (\d+.\d+)" -r '$1' | tr -d " ")
TIMES=~/repos/oort_ai/best.txt
best=$(rg "$SCENARIAO: (\d+.\d+)" $TIMES -r '$1')
if [ -z "$best" ]
then
    best="1000"
    echo "$SCENARIAO: $best" >> $TIMES
else
    echo "Best time: $best"
fi
if (( $(echo "$avg $best" | awk '{print ($1 < $2)}') )); then
    echo "New best time: $avg"
    sed -i "s/$SCENARIAO: $best/$SCENARIAO: $avg/" $TIMES
    git tag "$SCENARIAO_NAME-$avg"
fi
