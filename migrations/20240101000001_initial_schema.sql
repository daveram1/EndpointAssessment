-- Initial schema for Endpoint Assessment System

-- Endpoints table
CREATE TABLE endpoints (
    id UUID PRIMARY KEY,
    hostname VARCHAR(255) NOT NULL UNIQUE,
    os VARCHAR(100),
    os_version VARCHAR(100),
    agent_version VARCHAR(50),
    ip_addresses JSONB,
    last_seen TIMESTAMPTZ,
    status VARCHAR(20) DEFAULT 'offline',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_endpoints_hostname ON endpoints(hostname);
CREATE INDEX idx_endpoints_status ON endpoints(status);
CREATE INDEX idx_endpoints_last_seen ON endpoints(last_seen);

-- Server configuration
CREATE TABLE server_config (
    key VARCHAR(100) PRIMARY KEY,
    value TEXT NOT NULL
);

-- Check definitions
CREATE TABLE check_definitions (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    check_type VARCHAR(50) NOT NULL,
    parameters JSONB NOT NULL,
    severity VARCHAR(20) DEFAULT 'medium',
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_check_definitions_enabled ON check_definitions(enabled);
CREATE INDEX idx_check_definitions_check_type ON check_definitions(check_type);

-- Check results
CREATE TABLE check_results (
    id UUID PRIMARY KEY,
    endpoint_id UUID REFERENCES endpoints(id) ON DELETE CASCADE,
    check_id UUID REFERENCES check_definitions(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL,
    message TEXT,
    collected_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_check_results_endpoint_id ON check_results(endpoint_id);
CREATE INDEX idx_check_results_check_id ON check_results(check_id);
CREATE INDEX idx_check_results_collected_at ON check_results(collected_at);
CREATE INDEX idx_check_results_status ON check_results(status);

-- System snapshots
CREATE TABLE system_snapshots (
    id UUID PRIMARY KEY,
    endpoint_id UUID REFERENCES endpoints(id) ON DELETE CASCADE,
    cpu_usage REAL,
    memory_total BIGINT,
    memory_used BIGINT,
    disk_total BIGINT,
    disk_used BIGINT,
    processes JSONB,
    open_ports JSONB,
    installed_software JSONB,
    collected_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_system_snapshots_endpoint_id ON system_snapshots(endpoint_id);
CREATE INDEX idx_system_snapshots_collected_at ON system_snapshots(collected_at);

-- Admin users
CREATE TABLE admin_users (
    id UUID PRIMARY KEY,
    username VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(20) DEFAULT 'viewer',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_admin_users_username ON admin_users(username);
