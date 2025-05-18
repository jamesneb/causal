#!/bin/bash
set -e

# Build the Lambda extension binary
echo "Building Lambda extension..."
cd ../../../platforms/aws-lambda/extension
cargo build --release

# Create the layer structure
LAYER_DIR="$(mktemp -d)"
mkdir -p "${LAYER_DIR}/extensions"

# Copy the extension binary
cp ../../../target/release/causeway-lambda-extension "${LAYER_DIR}/extensions/causeway"
chmod +x "${LAYER_DIR}/extensions/causeway"

# Create ZIP file
LAYER_ZIP="../../../target/causeway-lambda-extension.zip"
cd "${LAYER_DIR}"
zip -r "${LAYER_ZIP}" .

echo "Layer created at ${LAYER_ZIP}"


