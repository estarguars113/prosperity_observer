#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

TARGET_DIR="$SCRIPT_DIR/target/azure"
mkdir -p "$TARGET_DIR"

echo "Building Rust custom handler for Azure Functions (x86_64-unknown-linux-musl)..."

# Install the musl target if not already present
rustup target add x86_64-unknown-linux-musl 2>/dev/null || true

# Build the function binary
cargo build --release --bin function --target x86_64-unknown-linux-musl

echo "Preparing function package..."

# Copy the binary with the expected name
cp "$SCRIPT_DIR/target/x86_64-unknown-linux-musl/release/function" "$TARGET_DIR/handler"

# Copy the indicators.json config into the package
cp "$SCRIPT_DIR/indicators.json" "$TARGET_DIR/indicators.json"

# Generate host.json for the custom handler
cat > "$TARGET_DIR/host.json" << 'EOF'
{
  "version": "2.0",
  "customHandler": {
    "description": {
      "defaultExecutablePath": "handler",
      "workingDirectory": "",
      "arguments": []
    },
    "enableForwardingHttpRequest": true
  },
  "logging": {
    "logLevel": {
      "default": "Information"
    }
  },
  "extensionBundle": {
    "id": "Microsoft.Azure.Functions.ExtensionBundle",
    "version": "[4.*, 5.0.0)"
  }
}
EOF

# Generate the Timer-trigger function definition
# The schedule is read from the "Schedule" app setting at runtime,
# so it can be changed without rebuilding the package.
mkdir -p "$TARGET_DIR/ProsperityExtract"

cat > "$TARGET_DIR/ProsperityExtract/function.json" << 'EOF'
{
  "bindings": [
    {
      "name": "timerTrigger",
      "type": "timerTrigger",
      "direction": "in",
      "schedule": "%Schedule%"
    }
  ]
}
EOF

# Package everything into a ZIP
cd "$TARGET_DIR"
zip -r function.zip handler indicators.json host.json ProsperityExtract/

echo "Azure Function package created: $TARGET_DIR/function.zip"
