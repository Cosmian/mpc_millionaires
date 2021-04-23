#!/bin/bash

# This scripts launches a git daemon which exposes the project
# 
# It is useful to locally test modifications using the UI on the CipherCompute EAP version
# 1. Make a change and test it using `./simulate.sh`
# 2. commit the change to the local git and note the git commit
# 3. Lauhch this script
# 4. Create/update a computation using the git URL above and the commit you want to test
# 5. Run the computation fron the UI

echo "starting a git daemon...ctrl+c to stop"
#get the iproute2 package 
docker exec runtime_0 /bin/bash -c "apt update && apt install iproute2" &> /dev/null
# grab host IP from comntainer
hostip=$(docker exec runtime_0 /bin/bash -c "ip route" | awk '/default/ {print $3}')
project=$(basename "$PWD")
echo "git URL: git://${hostip}:9418/${project}"
git daemon --reuseaddr --base-path=$(pwd)/.. --export-all --verbose $(pwd)/..
echo "git daemon stopped"