#!/bin/bash

# Colors for better output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Creating Rainbow-Lang Monorepo Structure...${NC}"

# Create base structure
mkdir -p {agent,backend,frontend,infra,tools,docs,examples,.github}

# Agent structure (Rust-focused)
mkdir -p agent/{core,platforms,collectors,packaging,common}
mkdir -p agent/core/{telemetry,sampling,transport,config}
mkdir -p agent/platforms/{aws-lambda,aws-ec2,aws-ecs,kubernetes}
mkdir -p agent/collectors/{http,aws-sdk,database,messaging,system}
mkdir -p agent/collectors/database/{dynamodb,rds}
mkdir -p agent/collectors/messaging/{sqs,sns}
mkdir -p agent/packaging/{lambda-layer,docker}

# Backend structure (Rust services)
mkdir -p backend/{api,ingestion,storage,correlation,analysis,services}
mkdir -p backend/api/{graphql,rest}
mkdir -p backend/ingestion/{receivers,processors,exporters}
mkdir -p backend/storage/{timeseries,metadata,state}
mkdir -p backend/correlation/{temporal,content,inference}
mkdir -p backend/analysis/{anomaly,patterns,metrics}
mkdir -p backend/services/{auth,notification,admin}

# Frontend (SvelteKit)
mkdir -p frontend/{static,src}
mkdir -p frontend/src/{lib,routes,components,stores}
mkdir -p frontend/src/components/{timeline,service-map,trace-viewer,state-viewer}
mkdir -p frontend/src/routes/{dashboard,traces,services,settings}

# Infrastructure
mkdir -p infra/{terraform,deployment,ci}
mkdir -p infra/terraform/{backend,storage,monitoring}
mkdir -p infra/deployment/{kubernetes,aws,docker-compose}

# Tools
mkdir -p tools/{build,dev,testing,release,scripts}
mkdir -p tools/scripts/{setup,development,deployment}

# Examples
mkdir -p examples/{aws-serverless,microservices,full-stack}

# Documentation
mkdir -p docs/{architecture,development,user,api}

# GitHub setup
mkdir -p .github/{workflows,ISSUE_TEMPLATE}

# Create initial Rust Cargo.toml files
echo -e "${YELLOW}Creating initial Rust project files...${NC}"

# Root Cargo.toml for workspace
cat > Cargo.toml << 'EOF'
[workspace]
members = [
    "agent/core/lib",
    "agent/common",
    "backend/api",
    "backend/ingestion",
    "backend/correlation",
]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1.28", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
anyhow = "1.0"
EOF

# Agent core Cargo.toml
mkdir -p agent/core/lib
cat > agent/core/lib/Cargo.toml << 'EOF'
[package]
name = "agent-core"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
EOF

mkdir -p agent/core/lib/src
cat > agent/core/lib/src/lib.rs << 'EOF'
pub mod telemetry;
pub mod config;
pub mod transport;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
EOF

# Create empty directories for Rust modules
mkdir -p agent/core/lib/src/{telemetry,config,transport}
touch agent/core/lib/src/telemetry/mod.rs
touch agent/core/lib/src/config/mod.rs
touch agent/core/lib/src/transport/mod.rs

# Common Rust library
mkdir -p agent/common/src
cat > agent/common/Cargo.toml << 'EOF'
[package]
name = "agent-common"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
EOF

cat > agent/common/src/lib.rs << 'EOF'
pub mod types;
pub mod utils;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
EOF

mkdir -p agent/common/src/{types,utils}
touch agent/common/src/types/mod.rs
touch agent/common/src/utils/mod.rs

# Setup frontend with SvelteKit
echo -e "${YELLOW}Setting up SvelteKit frontend skeleton...${NC}"

cat > frontend/package.json << 'EOF'
{
  "name": "causeway-frontend",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
    "check:watch": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch"
  },
  "devDependencies": {
    "@sveltejs/adapter-auto": "^2.0.0",
    "@sveltejs/kit": "^1.20.4",
    "svelte": "^4.0.5",
    "svelte-check": "^3.4.3",
    "tslib": "^2.4.1",
    "typescript": "^5.0.0",
    "vite": "^4.4.2"
  },
  "dependencies": {
    "d3": "^7.8.5"
  }
}
EOF

cat > frontend/tsconfig.json << 'EOF'
{
  "extends": "./.svelte-kit/tsconfig.json",
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "esModuleInterop": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "skipLibCheck": true,
    "sourceMap": true,
    "strict": true
  }
}
EOF

# Create sample SvelteKit component
mkdir -p frontend/src/lib/components
cat > frontend/src/lib/components/TimelineViewer.svelte << 'EOF'
<script lang="ts">
  export let data = [];
  export let width = 800;
  export let height = 200;
</script>

<div class="timeline-container" style="width: {width}px; height: {height}px;">
  <h3>Timeline Viewer</h3>
  <div class="timeline">
    <!-- Timeline visualization will go here -->
    <p>Sample timeline component</p>
  </div>
</div>

<style>
  .timeline-container {
    border: 1px solid #ccc;
    border-radius: 4px;
    padding: 1rem;
    margin: 1rem 0;
  }
  
  h3 {
    margin-top: 0;
  }
</style>
EOF

# Create a basic route
mkdir -p frontend/src/routes
cat > frontend/src/routes/+page.svelte << 'EOF'
<script lang="ts">
  import TimelineViewer from '$lib/components/TimelineViewer.svelte';
</script>

<h1>Causeway</h1>
<p>Distributed System Observability with Causal Inference</p>

<TimelineViewer />

<style>
  h1 {
    color: #3b82f6;
  }
</style>
EOF

# Create README
cat > README.md << 'EOF'
# Causeway - Distributed System Observability Platform

Causeway is a distributed system observability platform with state capture and causal inference.

## Components

- **Agent**: Lightweight instrumentation for AWS services
- **Backend**: Correlation and analysis engine
- **Frontend**: Developer-focused debugging UI

## Development

Use `just` to run common commands:

```bash
# List all available commands
just

# Build all components
just all

# Run in development mode
just dev
```

## Architecture

See `docs/architecture` for detailed design documentation.
EOF

# Create initial justfile
cat > justfile << 'EOF'
# List all available commands
default:
    @just --list

# Build all components
all: agent backend frontend

# Build the Rust agent
agent:
    cd agent && cargo build

# Build the Rust backend
backend:
    cd backend && cargo build

# Build the SvelteKit frontend
frontend:
    cd frontend && npm install && npm run build

# Run everything in development mode
dev:
    @echo "Starting development servers..."
    @echo "Use Ctrl+C to stop all processes"
    @tmux new-session -d -s dev "cd agent && cargo watch -x run || read"
    @tmux split-window -h "cd frontend && npm run dev || read"
    @tmux -2 attach-session -d

# Development mode for agent only
agent-dev:
    cd agent && cargo watch -x run

# Development mode for frontend only
frontend-dev:
    cd frontend && npm run dev

# Run tests
test:
    cd agent && cargo test
    cd backend && cargo test
    cd frontend && npm test

# Generate documentation
docs:
    cd agent && cargo doc --no-deps --open
    cd backend && cargo doc --no-deps --open

# Clean all build artifacts
clean:
    cd agent && cargo clean
    cd backend && cargo clean
    cd frontend && rm -rf node_modules .svelte-kit/build
EOF

# Create initial .gitignore
cat > .gitignore << 'EOF'
# Rust
/target/
**/target/
**/*.rs.bk
Cargo.lock

# Node.js
node_modules/
npm-debug.log*
yarn-debug.log*
yarn-error.log*
.pnpm-debug.log*

# SvelteKit
.svelte-kit/
build/
.env
.env.*
!.env.example

# IDE
.idea/
.vscode/
*.sublime-project
*.sublime-workspace

# OS
.DS_Store
Thumbs.db

# Misc
.tmp/
.cache/
EOF

# Create GitHub Actions workflow
mkdir -p .github/workflows
cat > .github/workflows/ci.yml << 'EOF'
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  rust:
    name: Rust
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - name: Check formatting
        run: cargo fmt --all -- --check
      - name: Clippy
        run: cargo clippy --all-targets -- -D warnings
      - name: Test
        run: cargo test

  frontend:
    name: Frontend
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: 18
          cache: 'npm'
          cache-dependency-path: frontend/package-lock.json
      - name: Install dependencies
        run: cd frontend && npm ci
      - name: Check
        run: cd frontend && npm run check
      - name: Build
        run: cd frontend && npm run build
EOF

# Initialize git repo
git init
git add .
git commit -m "Initial monorepo structure"

echo -e "${GREEN}Rainbow-Lang Monorepo structure created successfully!${NC}"
echo -e "${BLUE}Next steps:${NC}"
echo "1. Install just: cargo install just"
echo "2. Install cargo-watch for development: cargo install cargo-watch"
echo "3. Install tmux for dev mode: sudo apt install tmux (or brew install tmux on macOS)"
echo "4. Start development with: just dev"
echo -e "${YELLOW}Happy coding!${NC}"
