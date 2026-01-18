#!/bin/bash
export PATH="/home/lutica/.cargo/bin:$PATH" # 이렇게 lutica님의 경로를 직접 지정

# delette the static directory if it exists
if [ -d "./static" ]; then
    echo "Removing existing static directory..."
    rm -rf ./static
fi
mkdir ./static
cd ../rin_agent_front
npm run deploy:unix
cd ../rin_agent

# Find and forcefully kill existing rin_agent processes
pids=$(pgrep -f "./target/release/rin_agent")
if [ -n "$pids" ]; then
    echo "Killing existing rin_agent processes: $pids"
    kill -9 $pids
fi
# Build the project in release mode
echo "Building rin_agent with cargo..."
cargo +nightly build --release

# Start the server in the background with nohup
echo "Starting rin_agent with nohup..."
nohup ./target/release/rin_agent > rin_agent.out 2>&1 &

echo "rin_agent started."