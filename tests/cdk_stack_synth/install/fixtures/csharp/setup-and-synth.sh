#!/bin/bash
set -eu

"$CDK_PATH" synth $CDK_FLAGS --app 'dotnet run --project ./CSharp.csproj'
