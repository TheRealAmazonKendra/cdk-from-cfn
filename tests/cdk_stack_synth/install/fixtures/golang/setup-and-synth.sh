#!/bin/bash
set -eu

npm remove node_modules

WORK_DIR=$(pwd)
export GOMODCACHE="$PROJECT_ROOT/target/tmp/go-mod-cache"
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

"$CDK_PATH" synth $CDK_FLAGS --app "cd '$WORK_DIR' && go mod download && go run *.go" --output "$WORK_DIR/cdk.out"
