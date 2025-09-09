#!/bin/bash
set -eu

# Use shared node_modules from target/tmp
SHARED_NODE_MODULES="$PROJECT_ROOT/target/tmp/node_modules"

if [ ! -d "$SHARED_NODE_MODULES/aws-cdk-lib" ] || [ ! -d "$SHARED_NODE_MODULES/typescript" ] || [ ! -d "$SHARED_NODE_MODULES/ts-node" ]; then
    echo "Installing/updating shared node_modules at $SHARED_NODE_MODULES"
    mkdir -p "$PROJECT_ROOT/target/tmp"
    cp package.json "$PROJECT_ROOT/target/tmp/"
    cd "$PROJECT_ROOT/target/tmp"
    npm install --silent
    cd - > /dev/null
fi

# Create local symlink to shared node_modules if it doesn't exist
if [ ! -L "node_modules" ] && [ ! -d "node_modules" ]; then
    ln -s "$SHARED_NODE_MODULES" node_modules
fi

"$CDK_PATH" synth $CDK_FLAGS --app 'npx ts-node ./app.ts'
