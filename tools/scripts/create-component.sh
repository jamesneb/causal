#!/bin/bash
# save as tools/scripts/create-component.sh

set -e

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 COMPONENT_PATH COMPONENT_NAME"
    echo "Example: $0 agent/platforms/aws-ecs/collector ecs-collector"
    exit 1
fi

COMPONENT_PATH="$1"
COMPONENT_NAME="$2"
PROJECT_NAME=$(cat project-config.json | jq -r .name)

# Get relative path to core lib
CORE_PATH=$(realpath --relative-to="$COMPONENT_PATH" "agent/core/lib")
COMMON_PATH=$(realpath --relative-to="$COMPONENT_PATH" "agent/common")

echo "Creating component: $COMPONENT_NAME at $COMPONENT_PATH"
mkdir -p "$COMPONENT_PATH"

cargo generate --path tools/templates/component --name "$COMPONENT_NAME" \
  --define "project-name=$PROJECT_NAME" \
  --define "component-name=$COMPONENT_NAME" \
  --define "core-relative-path=$CORE_PATH" \
  --define "common-relative-path=$COMMON_PATH" \
  --destination "$COMPONENT_PATH"

echo "Component created successfully!"
