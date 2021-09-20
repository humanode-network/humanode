#!/bin/bash

# Set up commands.
command_to_run="cargo run"

# Set up timeout.
time_in_sec=60

# Build the node.
cargo build

# Run with empty state, then 2nd time with non-empty state.
for (( i=1; i <= 2; i++ ))
do

timeout $time_in_sec $command_to_run
exitcode=$?
echo "Status code of run $i is $exitcode"

# Check it's exit code. 
if [ $exitcode -ne 124 ] # 124 is the 'command times out' code.
then
echo "Error: app run $i doesn'n lasted for $time_in_sec seconds"
exit
fi

done

# TODO: Things to test 3
# TODO: Things to test 4

exit 0
