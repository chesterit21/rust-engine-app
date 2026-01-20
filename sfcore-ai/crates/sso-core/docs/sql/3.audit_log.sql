-- ============================================================================
-- SSO Fullstack Rust - PostgreSQL Migration Script
-- File: 003_audit_security_tables.sql
-- Description: Audit logging, security monitoring, and compliance tables
-- ============================================================================

-- ============================================================================
-- 1. AUDIT_LOGS TABLE (Comprehensive Audit Trail)
-- ============================================================================

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_user_id UUID NULL,
    action VARCHAR(100) NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    entity_id UUID NULL,
    old_values JSONB NULL,
    new_values JSONB NULL,
    ip_address INET NULL,
    user_agent TEXT NULL,
    request_id VARCHAR(255) NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Foreign keys (with SET NULL to preserve audit trail)
    CONSTRAINT fk_audit_logs_user 
        FOREIGN KEY (member_user_id) 
        REFERENCES member_user(id) 
        ON DELETE SET NULL
);

-- Indexes for querying audit logs
CREATE INDEX idx_audit_logs_user 
    ON audit_logs(member_user_id, created_at DESC);

CREATE INDEX idx_audit_logs_entity 
    ON audit_logs(entity_type, entity_id, created_at DESC);

CREATE INDEX idx_audit_logs_action 
    ON audit_logs(action, created_at DESC);

CREATE INDEX idx_audit_logs_created 
    ON audit_logs(created_at DESC);

CREATE INDEX idx_audit_logs_request 
    ON audit_logs(request_id) 
    WHERE request_id IS NOT NULL;

-- GIN index for JSONB searching
CREATE INDEX idx_audit_logs_old_values 
    ON audit_logs USING gin(old_values);

CREATE INDEX idx_audit_logs_new_values 
    ON audit_logs USING gin(new_values);

COMMENT ON TABLE audit_logs IS 'Comprehensive audit trail of all system actions';
COMMENT ON COLUMN audit_logs.action IS 'Action performed: login, logout, create, update, delete, etc.';
COMMENT ON COLUMN audit_logs.entity_type IS 'Type of entity affected (table name)';
COMMENT ON COLUMN audit_logs.old_values IS 'Previous values (for updates/deletes)';
COMMENT ON COLUMN audit_logs.new_values IS 'New values (for creates/updates)';
COMMENT ON COLUMN audit_logs.request_id IS 'Correlation ID for distributed tracing';

-- ============================================================================
-- 2. LOGIN_ATTEMPTS TABLE (Security Monitoring)
-- ============================================================================

CREATE TABLE login_attempts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) NOT NULL,
    ip_address INET NOT NULL,
    user_agent TEXT NULL,
    success BOOLEAN NOT NULL,
    failure_reason VARCHAR(100) NULL,
    location_country VARCHAR(2) NULL,
    location_city VARCHAR(100) NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Check constraints
    CONSTRAINT chk_login_failure_reason 
        CHECK (
            (success = TRUE AND failure_reason IS NULL) OR 
            (success = FALSE AND failure_reason IS NOT NULL)
        )
);

-- Indexes for security analysis
CREATE INDEX idx_login_attempts_email_time 
    ON login_attempts(email, created_at DESC);

CREATE INDEX idx_login_attempts_ip_time 
    ON login_attempts(ip_address, created_at DESC);

CREATE INDEX idx_login_attempts_success 
    ON login_attempts(success, created_at DESC);

CREATE INDEX idx_login_attempts_created 
    ON login_attempts(created_at DESC);

-- Composite index for rate limiting checks
CREATE INDEX idx_login_attempts_email_ip_recent 
    ON login_attempts(email, ip_address, created_at DESC) 
    WHERE created_at > NOW() - INTERVAL '1 hour';

COMMENT ON TABLE login_attempts IS 'Login attempt tracking for security monitoring and rate limiting';
COMMENT ON COLUMN login_attempts.failure_reason IS 'Reason: invalid_credentials, account_locked, email_not_verified, etc.';

-- ============================================================================
-- 3. SECURITY_EVENTS TABLE (Suspicious Activity Tracking)
-- ============================================================================

CREATE TABLE security_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_user_id UUID NULL,
    event_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    description TEXT NOT NULL,
    ip_address INET NULL,
    user_agent TEXT NULL,
    metadata JSONB NULL,
    resolved BOOLEAN NOT NULL DEFAULT FALSE,
    resolved_at TIMESTAMPTZ NULL,
    resolved_by UUID NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Foreign keys
    CONSTRAINT fk_security_events_user 
        FOREIGN KEY (member_user_id) 
        REFERENCES member_user(id) 
        ON DELETE SET NULL,
    
    CONSTRAINT fk_security_events_resolver 
        FOREIGN KEY (resolved_by) 
        REFERENCES member_user(id) 
        ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_severity CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT chk_event_type CHECK (event_type IN (
        'multiple_failed_logins',
        'suspicious_location',
        'password_reset_abuse',
        'token_theft_attempt',
        'concurrent_sessions',
        'privilege_escalation',
        'data_exfiltration',
        'brute_force_attack'
    ))
);

-- Indexes
CREATE INDEX idx_security_events_user 
    ON security_events(member_user_id, created_at DESC);

CREATE INDEX idx_security_events_type 
    ON security_events(event_type, severity, created_at DESC);

CREATE INDEX idx_security_events_unresolved 
    ON security_events(severity, created_at DESC) 
    WHERE resolved = FALSE;

CREATE INDEX idx_security_events_created 
    ON security_events(created_at DESC);

COMMENT ON TABLE security_events IS 'Security incidents and suspicious activity tracking';
COMMENT ON COLUMN security_events.event_type IS 'Type of security event detected';
COMMENT ON COLUMN security_events.severity IS 'Severity level: low, medium, high, critical';

-- ============================================================================
-- 4. IP_BLACKLIST TABLE (IP Blocking)
-- ============================================================================

CREATE TABLE ip_blacklist (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    ip_address INET NOT NULL UNIQUE,
    reason TEXT NOT NULL,
    blocked_until TIMESTAMPTZ NULL,
    is_permanent BOOLEAN NOT NULL DEFAULT FALSE,
    blocked_by UUID NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMPTZ NULL,
    
    -- Foreign keys
    CONSTRAINT fk_ip_blacklist_blocker 
        FOREIGN KEY (blocked_by) 
        REFERENCES member_user(id) 
        ON DELETE SET NULL
);

-- Indexes
CREATE INDEX idx_ip_blacklist_ip 
    ON ip_blacklist(ip_address) 
    WHERE blocked_until IS NULL OR blocked_until > NOW();

CREATE INDEX idx_ip_blacklist_expires 
    ON ip_blacklist(blocked_until) 
    WHERE blocked_until IS NOT NULL;

COMMENT ON TABLE ip_blacklist IS 'Blocked IP addresses for security';
COMMENT ON COLUMN ip_blacklist.blocked_until IS 'NULL for permanent blocks, timestamp for temporary blocks';

-- ============================================================================
-- 5. RATE_LIMITS TABLE (API Rate Limiting Tracking)
-- ============================================================================

CREATE TABLE rate_limits (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    identifier VARCHAR(255) NOT NULL,
    identifier_type VARCHAR(50) NOT NULL,
    endpoint VARCHAR(255) NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 1,
    window_start TIMESTAMPTZ NOT NULL,
    window_end TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Check constraints
    CONSTRAINT chk_identifier_type CHECK (identifier_type IN ('ip', 'user', 'api_key'))
);

-- Unique constraint for rate limit windows
CREATE UNIQUE INDEX idx_rate_limits_window 
    ON rate_limits(identifier, endpoint, window_start);

-- Indexes for cleanup
CREATE INDEX idx_rate_limits_window_end 
    ON rate_limits(window_end);

COMMENT ON TABLE rate_limits IS 'API rate limiting tracking (sliding window)';
COMMENT ON COLUMN rate_limits.identifier IS 'IP address, user ID, or API key';
COMMENT ON COLUMN rate_limits.identifier_type IS 'Type of identifier: ip, user, api_key';

-- ============================================================================
-- 6. API_KEYS TABLE (API Key Management for M2M Authentication)
-- ============================================================================

CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_user_id UUID NULL,
    client_app_id UUID NOT NULL,
    key_name VARCHAR(100) NOT NULL,
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    key_prefix VARCHAR(20) NOT NULL,
    scopes TEXT[] NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_used TIMESTAMPTZ NULL,
    expires_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NULL,
    revoked_at TIMESTAMPTZ NULL,
    revoked_by UUID NULL,
    
    -- Foreign keys
    CONSTRAINT fk_api_keys_user 
        FOREIGN KEY (member_user_id) 
        REFERENCES member_user(id) 
        ON DELETE CASCADE,
    
    CONSTRAINT fk_api_keys_client 
        FOREIGN KEY (client_app_id) 
        REFERENCES client_app(id) 
        ON DELETE CASCADE,
    
    CONSTRAINT fk_api_keys_creator 
        FOREIGN KEY (created_by) 
        REFERENCES member_user(id) 
        ON DELETE SET NULL,
    
    CONSTRAINT fk_api_keys_revoker 
        FOREIGN KEY (revoked_by) 
        REFERENCES member_user(id) 
        ON DELETE SET NULL
);

-- Indexes
CREATE INDEX idx_api_keys_hash 
    ON api_keys(key_hash) 
    WHERE is_active = TRUE AND revoked_at IS NULL;

CREATE INDEX idx_api_keys_prefix 
    ON api_keys(key_prefix) 
    WHERE is_active = TRUE;

CREATE INDEX idx_api_keys_user 
    ON api_keys(member_user_id) 
    WHERE is_active = TRUE;

CREATE INDEX idx_api_keys_client 
    ON api_keys(client_app_id) 
    WHERE is_active = TRUE;

CREATE INDEX idx_api_keys_expires 
    ON api_keys(expires_at) 
    WHERE expires_at IS NOT NULL AND is_active = TRUE;

COMMENT ON TABLE api_keys IS 'API keys for machine-to-machine authentication';
COMMENT ON COLUMN api_keys.key_hash IS 'SHA256 hash of the API key';
COMMENT ON COLUMN api_keys.key_prefix IS 'First 8 chars of key for identification (e.g., sk_live_...)';
COMMENT ON COLUMN api_keys.scopes IS 'Array of permission scopes';

-- ============================================================================
-- 7. WEBHOOKS TABLE (Webhook Configuration)
-- ============================================================================

CREATE TABLE webhooks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    client_app_id UUID NOT NULL,
    url TEXT NOT NULL,
    secret VARCHAR(255) NOT NULL,
    events TEXT[] NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_triggered TIMESTAMPTZ NULL,
    failure_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NULL,
    modified_at TIMESTAMPTZ NULL,
    
    -- Foreign keys
    CONSTRAINT fk_webhooks_client 
        FOREIGN KEY (client_app_id) 
        REFERENCES client_app(id) 
        ON DELETE CASCADE,
    
    CONSTRAINT fk_webhooks_creator 
        FOREIGN KEY (created_by) 
        REFERENCES member_user(id) 
        ON DELETE SET NULL
);

-- Indexes
CREATE INDEX idx_webhooks_client 
    ON webhooks(client_app_id) 
    WHERE is_active = TRUE;

CREATE INDEX idx_webhooks_events 
    ON webhooks USING gin(events) 
    WHERE is_active = TRUE;

COMMENT ON TABLE webhooks IS 'Webhook endpoints for event notifications';
COMMENT ON COLUMN webhooks.events IS 'Array of event types to trigger webhook';
COMMENT ON COLUMN webhooks.secret IS 'Secret for HMAC signature verification';

-- ============================================================================
-- 8. WEBHOOK_DELIVERIES TABLE (Webhook Delivery Log)
-- ============================================================================

CREATE TABLE webhook_deliveries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    webhook_id UUID NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    http_status INTEGER NULL,
    response_body TEXT NULL,
    response_time_ms INTEGER NULL,
    success BOOLEAN NOT NULL DEFAULT FALSE,
    error_message TEXT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0,
    next_retry_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Foreign keys
    CONSTRAINT fk_webhook_deliveries_webhook 
        FOREIGN KEY (webhook_id) 
        REFERENCES webhooks(id) 
        ON DELETE CASCADE
);

-- Indexes
CREATE INDEX idx_webhook_deliveries_webhook 
    ON webhook_deliveries(webhook_id, created_at DESC);

CREATE INDEX idx_webhook_deliveries_retry 
    ON webhook_deliveries(next_retry_at) 
    WHERE next_retry_at IS NOT NULL AND success = FALSE;

CREATE INDEX idx_webhook_deliveries_created 
    ON webhook_deliveries(created_at DESC);

COMMENT ON TABLE webhook_deliveries IS 'Webhook delivery attempts and results';
COMMENT ON COLUMN webhook_deliveries.retry_count IS 'Number of retry attempts';

-- ============================================================================
-- End of 003_audit_security_tables.sql
-- ============================================================================