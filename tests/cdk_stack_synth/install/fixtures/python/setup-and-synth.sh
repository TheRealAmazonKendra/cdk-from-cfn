#!/bin/bash
set -eu

# Use absolute path to avoid creating target directory in testing/
SHARED_VENV="$PROJECT_ROOT/target/tmp/.shared-python-test-venv"
REQUIREMENTS_HASH="$PROJECT_ROOT/target/tmp/.python-requirements-hash"

# Calculate current requirements hash
if command -v md5 >/dev/null 2>&1; then
    CURRENT_HASH=$(md5 -q requirements.txt)
else
    CURRENT_HASH=$(md5sum requirements.txt | cut -d' ' -f1)
fi

if [ ! -d "$SHARED_VENV" ] || [ ! -f "$REQUIREMENTS_HASH" ] || [ "$(cat "$REQUIREMENTS_HASH" 2>/dev/null)" != "$CURRENT_HASH" ]; then
    echo "Creating/updating shared venv"
    rm -rf "$SHARED_VENV"
    python3 -m venv "$SHARED_VENV"
    source "$SHARED_VENV/bin/activate"
    pip install --disable-pip-version-check -q -r requirements.txt
    echo "$CURRENT_HASH" > "$REQUIREMENTS_HASH"
else
    echo "Using existing shared venv"
    source "$SHARED_VENV/bin/activate"
fi

CDK_DISABLE_VERSION_CHECK=1 "$CDK_PATH" synth $CDK_FLAGS --app 'python3 app.py'
