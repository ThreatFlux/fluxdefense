.PHONY: help start stop start-backend start-frontend stop-backend stop-frontend status build build-backend build-frontend clean logs logs-backend logs-frontend dev prod install-deps

# Default target
help:
	@echo "FluxDefense - Makefile Commands"
	@echo "==============================="
	@echo ""
	@echo "Quick Start:"
	@echo "  make start          - Start both backend and frontend"
	@echo "  make stop           - Stop both backend and frontend"
	@echo "  make status         - Check status of all services"
	@echo "  make logs           - View logs for both services"
	@echo ""
	@echo "Individual Services:"
	@echo "  make start-backend  - Start only the backend server"
	@echo "  make start-frontend - Start only the frontend development server"
	@echo "  make stop-backend   - Stop only the backend server"
	@echo "  make stop-frontend  - Stop only the frontend server"
	@echo ""
	@echo "Build Commands:"
	@echo "  make build          - Build both backend and frontend"
	@echo "  make build-backend  - Build backend (release mode)"
	@echo "  make build-frontend - Build frontend for production"
	@echo ""
	@echo "Development:"
	@echo "  make dev            - Start in development mode with hot reload"
	@echo "  make prod           - Build and start in production mode"
	@echo "  make install-deps   - Install all dependencies"
	@echo ""
	@echo "Maintenance:"
	@echo "  make clean          - Clean all build artifacts"
	@echo "  make logs-backend   - View backend logs"
	@echo "  make logs-frontend  - View frontend logs"

# PID file locations
BACKEND_PID = /tmp/fluxdefense-backend.pid
FRONTEND_PID = /tmp/fluxdefense-frontend.pid

# Log file locations
BACKEND_LOG = /tmp/fluxdefense-backend.log
FRONTEND_LOG = /tmp/fluxdefense-frontend.log

# Start all services
start: start-backend start-frontend
	@echo "✅ FluxDefense services started successfully"
	@echo "   Backend:  http://localhost:8080"
	@echo "   Frontend: http://localhost:5173"

# Stop all services
stop: stop-frontend stop-backend
	@echo "✅ All FluxDefense services stopped"

# Start backend
start-backend:
	@if [ -f $(BACKEND_PID) ] && kill -0 `cat $(BACKEND_PID)` 2>/dev/null; then \
		echo "⚠️  Backend is already running (PID: `cat $(BACKEND_PID)`)"; \
	else \
		echo "🚀 Starting FluxDefense backend..."; \
		RUST_LOG=info nohup cargo run --release > $(BACKEND_LOG) 2>&1 & echo $$! > $(BACKEND_PID); \
		sleep 2; \
		if kill -0 `cat $(BACKEND_PID)` 2>/dev/null; then \
			echo "✅ Backend started (PID: `cat $(BACKEND_PID)`)"; \
		else \
			echo "❌ Backend failed to start. Check logs: make logs-backend"; \
			rm -f $(BACKEND_PID); \
			exit 1; \
		fi \
	fi

# Start frontend
start-frontend:
	@if [ -f $(FRONTEND_PID) ] && kill -0 `cat $(FRONTEND_PID)` 2>/dev/null; then \
		echo "⚠️  Frontend is already running (PID: `cat $(FRONTEND_PID)`)"; \
	else \
		echo "🚀 Starting FluxDefense frontend..."; \
		cd web-dashboard && nohup npm run dev > $(FRONTEND_LOG) 2>&1 & echo $$! > $(FRONTEND_PID); \
		sleep 3; \
		if kill -0 `cat $(FRONTEND_PID)` 2>/dev/null; then \
			echo "✅ Frontend started (PID: `cat $(FRONTEND_PID)`)"; \
		else \
			echo "❌ Frontend failed to start. Check logs: make logs-frontend"; \
			rm -f $(FRONTEND_PID); \
			exit 1; \
		fi \
	fi

# Stop backend
stop-backend:
	@if [ -f $(BACKEND_PID) ]; then \
		if kill -0 `cat $(BACKEND_PID)` 2>/dev/null; then \
			echo "🛑 Stopping backend (PID: `cat $(BACKEND_PID)`)..."; \
			kill `cat $(BACKEND_PID)`; \
			rm -f $(BACKEND_PID); \
			echo "✅ Backend stopped"; \
		else \
			echo "⚠️  Backend not running, cleaning up PID file"; \
			rm -f $(BACKEND_PID); \
		fi \
	else \
		echo "ℹ️  Backend is not running"; \
	fi

# Stop frontend
stop-frontend:
	@if [ -f $(FRONTEND_PID) ]; then \
		if kill -0 `cat $(FRONTEND_PID)` 2>/dev/null; then \
			echo "🛑 Stopping frontend (PID: `cat $(FRONTEND_PID)`)..."; \
			kill `cat $(FRONTEND_PID)`; \
			rm -f $(FRONTEND_PID); \
			echo "✅ Frontend stopped"; \
		else \
			echo "⚠️  Frontend not running, cleaning up PID file"; \
			rm -f $(FRONTEND_PID); \
		fi \
	else \
		echo "ℹ️  Frontend is not running"; \
	fi

# Check status of all services
status:
	@echo "FluxDefense Service Status"
	@echo "=========================="
	@if [ -f $(BACKEND_PID) ] && kill -0 `cat $(BACKEND_PID)` 2>/dev/null; then \
		echo "✅ Backend:  Running (PID: `cat $(BACKEND_PID)`)"; \
	else \
		echo "❌ Backend:  Not running"; \
	fi
	@if [ -f $(FRONTEND_PID) ] && kill -0 `cat $(FRONTEND_PID)` 2>/dev/null; then \
		echo "✅ Frontend: Running (PID: `cat $(FRONTEND_PID)`)"; \
	else \
		echo "❌ Frontend: Not running"; \
	fi
	@echo ""
	@echo "Service URLs:"
	@echo "  Backend API: http://localhost:8080"
	@echo "  Frontend UI: http://localhost:5173"

# Build both backend and frontend
build: build-backend build-frontend
	@echo "✅ Build complete"

# Build backend
build-backend:
	@echo "🔨 Building backend..."
	cargo build --release
	@echo "✅ Backend built successfully"

# Build frontend
build-frontend:
	@echo "🔨 Building frontend..."
	cd web-dashboard && npm run build
	@echo "✅ Frontend built successfully"

# Development mode - start with hot reload
dev:
	@echo "🚀 Starting in development mode..."
	@make stop > /dev/null 2>&1 || true
	@echo "Starting backend with hot reload..."
	RUST_LOG=debug cargo watch -x run > $(BACKEND_LOG) 2>&1 & echo $$! > $(BACKEND_PID)
	@echo "Starting frontend with hot reload..."
	cd web-dashboard && npm run dev > $(FRONTEND_LOG) 2>&1 & echo $$! > $(FRONTEND_PID)
	@sleep 3
	@make status

# Production mode - build and start optimized versions
prod: build
	@echo "🚀 Starting in production mode..."
	@make stop > /dev/null 2>&1 || true
	RUST_LOG=info ./target/release/fluxdefense > $(BACKEND_LOG) 2>&1 & echo $$! > $(BACKEND_PID)
	cd web-dashboard && npm run preview > $(FRONTEND_LOG) 2>&1 & echo $$! > $(FRONTEND_PID)
	@sleep 3
	@make status

# Install all dependencies
install-deps:
	@echo "📦 Installing dependencies..."
	@echo "Installing Rust dependencies..."
	cargo fetch
	@echo "Installing frontend dependencies..."
	cd web-dashboard && npm install
	@echo "✅ All dependencies installed"

# View logs
logs: logs-backend logs-frontend

# View backend logs
logs-backend:
	@if [ -f $(BACKEND_LOG) ]; then \
		echo "📜 Backend logs (last 50 lines):"; \
		echo "================================"; \
		tail -n 50 $(BACKEND_LOG); \
	else \
		echo "ℹ️  No backend logs found"; \
	fi

# View frontend logs
logs-frontend:
	@if [ -f $(FRONTEND_LOG) ]; then \
		echo "📜 Frontend logs (last 50 lines):"; \
		echo "================================="; \
		tail -n 50 $(FRONTEND_LOG); \
	else \
		echo "ℹ️  No frontend logs found"; \
	fi

# Clean build artifacts and logs
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	cd web-dashboard && rm -rf dist node_modules
	rm -f $(BACKEND_PID) $(FRONTEND_PID) $(BACKEND_LOG) $(FRONTEND_LOG)
	@echo "✅ Clean complete"

# Restart all services
restart: stop start
	@echo "✅ Services restarted"

# Health check
health:
	@echo "🏥 Checking service health..."
	@echo -n "Backend API: "
	@curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/api/health || echo "000"
	@echo ""
	@echo -n "Frontend: "
	@curl -s -o /dev/null -w "%{http_code}" http://localhost:5173 || echo "000"
	@echo ""

# Watch logs in real-time
watch-logs:
	@echo "📜 Watching logs (Ctrl+C to stop)..."
	@echo "===================================="
	tail -f $(BACKEND_LOG) $(FRONTEND_LOG) 2>/dev/null || echo "No logs available yet"