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

dev-backend: cd backend && cargo watch -x run

dev-frontend: 
    cd frontend && pnpm dev 

build-lambda-layer:
    cd agent && cargo lambda build --release --output-format zip

test-all:
    cd agent && cargo test
    cd backend && cargo test
    cd frontend && pnpm test
# Add these to your justfile

# Update project name
rename-project name:
    @echo "Updating project name to: {{name}}"
    @echo '{"name": "{{name}}", "version": "0.1.0"}' > project-config.json
    @./tools/scripts/sync-project-config.py
    @echo "Project renamed successfully!"

# Create a new component
create-component path name:
    @./tools/scripts/create-component.sh {{path}} {{name}}
