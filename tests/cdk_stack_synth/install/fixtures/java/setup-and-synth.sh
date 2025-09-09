#!/bin/bash
set -eu

# Use shared Maven repository
export MAVEN_OPTS="-Dmaven.repo.local=$PROJECT_ROOT/target/tmp/.m2/repository"

"$CDK_PATH" synth $CDK_FLAGS --app 'mvn -e -q compile exec:java'
