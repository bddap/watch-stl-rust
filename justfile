# https://github.com/casey/just

# repeatedly out the stl every few seconds for testing
exercise:
    #!/bin/bash
    set -ueo pipefail
    delay=1
    mkdir -p tmp
    while true; do
       for mesh in $(ls samples | grep '.stl'); do
          cp samples/"$mesh" tmp/test.stl
          sleep $delay
          cat samples/"$mesh" > tmp/test.stl
          sleep $delay
       done
    done

# build and run in debug, run this at the same time as "exercise"
run:
    cargo run -- tmp/test.stl
