# Endpoint Assessment System

A Rust-based client/server system for endpoint monitoring and security assessment. Deploy agents on your endpoints to collect system information, execute custom compliance checks, and report results to a central server with a web dashboard.

## Features

- **Agent-based monitoring**: Lightweight agent collects system metrics and executes checks
- **Custom check definitions**: Define checks via the web UI or API
- **7 check types**: File existence, file content, registry keys (Windows), config settings, running processes, open ports, command output
- **Web dashboard**: Real-time overview of endpoint status and check results
- **REST API**: Full API for integration with other tools
- **Cross-platform**: Agents run on Linux and Windows

## Architecture

```
┌─────────────┐     HTTPS      ┌─────────────────────────────────┐
│   Agent     │◄──────────────►│            Server               │
│  (endpoint) │                │  ┌─────────┐  ┌──────────────┐  │
└─────────────┘                │  │ REST API│  │   Web UI     │  │
                               │  └────┬────┘  └──────┬───────┘  │
┌─────────────┐                │       │              │          │
│   Agent     │◄──────────────►│  ┌────▼──────────────▼───────┐  │
│  (endpoint) │                │  │       PostgreSQL          │  │
└─────────────┘                │  └───────────────────────────┘  │
                               └─────────────────────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- PostgreSQL 14+

### Server Setup

1. **Create the database:**
   ```bash
   createdb endpoint_assessment
   ```

2. **Configure environment:**
   ```bash
   cp .env.example .env
   # Edit .env with your database URL and secrets
   ```

3. **Run the server:**
   ```bash
   cargo run -p server
   ```

4. **Create admin user:**

   Visit `http://localhost:8080/setup` to create the initial admin account.

### Agent Setup

Run the agent on each endpoint you want to monitor:

```bash
cargo run -p agent -- http://your-server:8080 your-agent-secret
```

Or set environment variables:
```bash
export SERVER_URL=http://your-server:8080
export AGENT_SECRET=your-agent-secret
cargo run -p agent
```

## Configuration

### Server Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | (required) |
| `HOST` | Server bind address | `0.0.0.0` |
| `PORT` | Server port | `8080` |
| `AGENT_SECRET` | Shared secret for agent auth | `change-me-in-production` |
| `SESSION_SECRET` | Secret for session cookies | `session-secret-change-me` |
| `OFFLINE_THRESHOLD_MINUTES` | Minutes before marking endpoint offline | `10` |

### Agent Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SERVER_URL` | Server URL | (required) |
| `AGENT_SECRET` | Shared secret matching server | (required) |
| `COLLECTION_INTERVAL_SECS` | Seconds between collection cycles | `300` |
| `HOSTNAME_OVERRIDE` | Override detected hostname | (auto-detect) |

## Check Types

Define checks in the web UI at `/checks/new` or via the API.

### file_exists
Check if a file exists at the specified path.
```json
{"path": "/etc/passwd"}
```

### file_content
Check if file content matches (or doesn't match) a regex pattern.
```json
{
  "path": "/etc/ssh/sshd_config",
  "pattern": "PermitRootLogin no",
  "should_match": true
}
```

### registry_key (Windows only)
Check Windows registry key existence or value.
```json
{
  "path": "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion",
  "value_name": "ProgramFilesDir",
  "expected": "C:\\Program Files"
}
```

### config_setting
Check key=value in configuration files.
```json
{
  "file": "/etc/myapp/config.ini",
  "key": "max_connections",
  "expected": "100"
}
```

### process_running
Check if a process is running.
```json
{"name": "nginx"}
```

### port_open
Check if a port is listening.
```json
{"port": 443}
```

### command_output
Execute a command and check output against a pattern.
```json
{
  "command": "uname -r",
  "expected_pattern": "^6\\."
}
```

## API Reference

### Agent API
All agent endpoints require `X-Agent-Secret` header.

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/agent/register` | Register new endpoint |
| POST | `/api/agent/heartbeat` | Send heartbeat with system snapshot |
| GET | `/api/agent/checks` | Get assigned check definitions |
| POST | `/api/agent/results` | Submit check results |

### Admin API

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/endpoints` | List all endpoints |
| GET | `/api/endpoints/{id}` | Get endpoint details |
| DELETE | `/api/endpoints/{id}` | Remove endpoint |
| GET | `/api/checks` | List check definitions |
| POST | `/api/checks` | Create check definition |
| PUT | `/api/checks/{id}` | Update check definition |
| DELETE | `/api/checks/{id}` | Delete check definition |
| GET | `/api/results` | Query check results |
| GET | `/api/reports/summary` | Dashboard summary data |

## Web UI

| Route | Description |
|-------|-------------|
| `/` | Dashboard with status overview |
| `/endpoints` | Endpoint list and management |
| `/endpoints/{id}` | Endpoint detail view |
| `/checks` | Check definition management |
| `/reports` | Reporting and statistics |
| `/login` | Admin login |
| `/setup` | Initial admin user creation |

## Project Structure

```
EndpointAssessment/
├── common/           # Shared library crate
│   └── src/
│       ├── models.rs     # Data types
│       ├── protocol.rs   # API request/response types
│       └── checks.rs     # Check type definitions
├── server/           # Server binary crate
│   └── src/
│       ├── api/          # REST API handlers
│       ├── db/           # Database operations
│       ├── web/          # Web UI routes & templates
│       └── services/     # Background tasks
├── agent/            # Agent binary crate
│   └── src/
│       ├── collectors/   # System info collection
│       ├── checks/       # Check execution engine
│       └── client.rs     # Server communication
└── migrations/       # PostgreSQL migrations
```

## Security Considerations

- Change default secrets (`AGENT_SECRET`, `SESSION_SECRET`) in production
- Use HTTPS in production (place behind a reverse proxy like nginx)
- The agent executes `command_output` checks - ensure check definitions are trusted
- Registry checks only work on Windows; they're skipped on other platforms

## License

MIT
