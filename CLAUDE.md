# FluxDefense - Claude Code Guide

## Quick Start for Development

### 🚀 Starting the Application

```bash
# Start both backend and frontend
make start

# Check if services are running
make status

# View logs if something goes wrong
make logs
```

The application will be available at:
- **Frontend**: http://localhost:5173
- **Backend API**: http://localhost:8080

### 🛠️ Development Workflow

```bash
# Install all dependencies first
make install-deps

# Start in development mode with hot reload
make dev

# Watch logs in real-time while coding
make watch-logs

# Check service health
make health
```

### 🐛 Debugging Tips

1. **Backend Issues**:
   ```bash
   # View backend logs
   make logs-backend
   
   # Restart just the backend
   make stop-backend && make start-backend
   
   # Run with debug logging
   RUST_LOG=debug make start-backend
   ```

2. **Frontend Issues**:
   ```bash
   # View frontend logs
   make logs-frontend
   
   # Restart just the frontend
   make stop-frontend && make start-frontend
   ```

3. **Clean Start**:
   ```bash
   # Stop everything and clean artifacts
   make stop && make clean
   
   # Fresh build and start
   make build && make start
   ```

## Project Structure

```
fluxdefense/
├── src/                    # Rust backend source
│   ├── main.rs            # Entry point
│   ├── api/               # REST API endpoints
│   ├── linux_security/    # Linux security monitoring
│   └── system_metrics.rs  # System monitoring
├── web-dashboard/         # React frontend
│   ├── src/
│   │   ├── components/    # UI components
│   │   └── services/      # API client
│   └── dist/             # Production build
├── file-scanner/         # Binary analysis tool
└── Makefile             # Project commands
```

## Common Tasks

### Running Tests
```bash
# Backend tests
cargo test

# Frontend tests
cd web-dashboard && npm test

# Integration tests
./scripts/test_phase2_integration.sh
```

### Building for Production
```bash
# Build and start in production mode
make prod

# Just build without starting
make build
```

### Monitoring System Performance
```bash
# Real-time system metrics (Linux)
./scripts/show_system_tray_demo.sh

# Memory usage display
./scripts/show_memory_display_demo.sh
```

### API Endpoints

The backend provides these main endpoints:

- `GET /api/health` - Health check
- `GET /api/system/stats` - System statistics
- `GET /api/processes` - Running processes
- `GET /api/network/connections` - Network connections
- `GET /api/security/events` - Security events
- `WS /ws` - WebSocket for real-time updates

### WebSocket Events

Connect to `ws://localhost:8080/ws` for real-time updates:

```javascript
// Example WebSocket connection
const ws = new WebSocket('ws://localhost:8080/ws');
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Event type:', data.event_type);
};
```

## Troubleshooting

### Port Already in Use
```bash
# Find and kill processes on ports
lsof -ti:8080 | xargs kill -9  # Backend port
lsof -ti:5173 | xargs kill -9  # Frontend port
```

### Backend Won't Start
```bash
# Check Rust compilation
cargo check

# View detailed logs
tail -f /tmp/fluxdefense-backend.log
```

### Frontend Build Issues
```bash
# Clear npm cache
cd web-dashboard
npm cache clean --force
rm -rf node_modules package-lock.json
npm install
```

### Permission Issues (Linux)
```bash
# The backend needs elevated permissions for security monitoring
sudo make start-backend

# Or run without advanced features
DISABLE_SECURITY_FEATURES=1 make start
```

## Development Best Practices

1. **Always check service status** before making changes:
   ```bash
   make status
   ```

2. **Use development mode** for faster iteration:
   ```bash
   make dev
   ```

3. **Monitor logs** in a separate terminal:
   ```bash
   make watch-logs
   ```

4. **Clean restart** when switching branches:
   ```bash
   make restart
   ```

5. **Test API endpoints** while developing:
   ```bash
   # Test backend health
   curl http://localhost:8080/api/health
   
   # Test WebSocket
   websocat ws://localhost:8080/ws
   ```

## Quick Commands Reference

| Command | Description |
|---------|-------------|
| `make start` | Start all services |
| `make stop` | Stop all services |
| `make restart` | Restart services |
| `make status` | Check service status |
| `make logs` | View recent logs |
| `make dev` | Development mode |
| `make prod` | Production mode |
| `make build` | Build everything |
| `make clean` | Clean artifacts |
| `make health` | Health check |

## File Scanner Integration

The file-scanner tool can be used for binary analysis:

```bash
# Build file scanner
cd file-scanner && cargo build --release

# Scan a suspicious file
./file-scanner/target/release/file-scanner /path/to/file --format yaml

# Use as MCP tool
./file-scanner/target/release/file-scanner mcp-stdio
```

See `file-scanner/CLAUDE.md` for detailed file scanner usage.