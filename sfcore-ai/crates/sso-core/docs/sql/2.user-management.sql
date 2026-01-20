-- ============================================================================
-- SSO Fullstack Rust - PostgreSQL Migration Script
-- File: 002_authentication_tables.sql
-- Description: Authentication, Session, and OAuth token management tables
-- ============================================================================

-- ============================================================================
-- 1. USER_SESSIONS TABLE (Active User Sessions)
-- ============================================================================

CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_user_id UUID NOT NULL,
    session_token VARCHAR(255) NOT NULL UNIQUE,
    ip_address INET NULL,
    user_agent TEXT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    last_activity TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Foreign keys
    CONSTRAINT fk_user_sessions_user 
        FOREIGN KEY (member_user_id) 
        REFERENCES member_user(id) 
        ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX idx_user_sessions_user 
    ON user_sessions(member_user_id) 
    WHERE is_active = TRUE;

CREATE INDEX idx_user_sessions_token 
    ON user_sessions(session_token) 
    WHERE is_active = TRUE;

CREATE INDEX idx_user_sessions_expires 
    ON user_sessions(expires_at) 
    WHERE is_active = TRUE;

CREATE INDEX idx_user_sessions_activity 
    ON user_sessions(last_activity DESC) 
    WHERE is_active = TRUE;

COMMENT ON TABLE user_sessions IS 'Active user session management';
COMMENT ON COLUMN user_sessions.session_token IS 'Unique session identifier (stored in cookie)';
COMMENT ON COLUMN user_sessions.expires_at IS 'Session expiration timestamp';

-- ============================================================================
-- 2. REFRESH_TOKENS TABLE (JWT Refresh Token Management)
-- ============================================================================

CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_user_id UUID NOT NULL,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ NULL,
    replaced_by UUID NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by_ip INET NULL,
    
    -- Foreign keys
    CONSTRAINT fk_refresh_tokens_user 
        FOREIGN KEY (member_user_id) 
        REFERENCES member_user(id) 
        ON DELETE CASCADE,
    
    CONSTRAINT fk_refresh_tokens_replaced 
        FOREIGN KEY (replaced_by) 
        REFERENCES refresh_tokens(id) 
        ON DELETE SET NULL
);

-- Indexes
CREATE INDEX idx_refresh_tokens_user 
    ON refresh_tokens(member_user_id) 
    WHERE revoked_at IS NULL;

CREATE INDEX idx_refresh_tokens_hash 
    ON refresh_tokens(token_hash) 
    WHERE revoked_at IS NULL;

CREATE INDEX idx_refresh_tokens_expires 
    ON refresh_tokens(expires_at) 
    WHERE revoked_at IS NULL;

CREATE INDEX idx_refresh_tokens_chain 
    ON refresh_tokens(replaced_by) 
    WHERE replaced_by IS NOT NULL;

COMMENT ON TABLE refresh_tokens IS 'JWT refresh token management with rotation support';
COMMENT ON COLUMN refresh_tokens.token_hash IS 'SHA256 hash of the refresh token';
COMMENT ON COLUMN refresh_tokens.replaced_by IS 'Token rotation tracking (points to new token)';

-- ============================================================================
-- 3. ACCESS_TOKENS TABLE (JWT Access Token Tracking - Optional)
-- ============================================================================

CREATE TABLE access_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    jti VARCHAR(255) NOT NULL UNIQUE,
    member_user_id UUID NOT NULL,
    token_type VARCHAR(20) NOT NULL DEFAULT 'Bearer',
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ NULL,
    scopes TEXT[] NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Foreign keys
    CONSTRAINT fk_access_tokens_user 
        FOREIGN KEY (member_user_id) 
        REFERENCES member_user(id) 
        ON DELETE CASCADE
);

-- Indexes
CREATE INDEX idx_access_tokens_jti 
    ON access_tokens(jti) 
    WHERE revoked_at IS NULL;

CREATE INDEX idx_access_tokens_user 
    ON access_tokens(member_user_id) 
    WHERE revoked_at IS NULL;

CREATE INDEX idx_access_tokens_expires 
    ON access_tokens(expires_at);

COMMENT ON TABLE access_tokens IS 'JWT access token tracking for revocation support';
COMMENT ON COLUMN access_tokens.jti IS 'JWT ID - unique identifier for the token';
COMMENT ON COLUMN access_tokens.scopes IS 'Array of permission scopes granted to this token';

-- ============================================================================
-- 4. OAUTH_PROVIDERS TABLE (External OAuth Provider Accounts)
-- ============================================================================

CREATE TABLE oauth_providers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_user_id UUID NOT NULL,
    provider VARCHAR(50) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    provider_email VARCHAR(255) NULL,
    provider_name VARCHAR(255) NULL,
    provider_avatar TEXT NULL,
    access_token TEXT NULL,
    refresh_token TEXT NULL,
    token_expires_at TIMESTAMPTZ NULL,
    last_login TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMPTZ NULL,
    
    -- Foreign keys
    CONSTRAINT fk_oauth_providers_user 
        FOREIGN KEY (member_user_id) 
        REFERENCES member_user(id) 
        ON DELETE CASCADE,
    
    -- Check constraints
    CONSTRAINT chk_oauth_provider CHECK (provider IN ('google', 'facebook', 'microsoft', 'github', 'apple'))
);

-- Unique constraints
CREATE UNIQUE INDEX idx_oauth_providers_provider_user 
    ON oauth_providers(provider, provider_user_id);

CREATE UNIQUE INDEX idx_oauth_providers_user_provider 
    ON oauth_providers(member_user_id, provider);

-- Performance indexes
CREATE INDEX idx_oauth_providers_user 
    ON oauth_providers(member_user_id);

CREATE INDEX idx_oauth_providers_email 
    ON oauth_providers(provider_email) 
    WHERE provider_email IS NOT NULL;

COMMENT ON TABLE oauth_providers IS 'External OAuth provider accounts linked to users';
COMMENT ON COLUMN oauth_providers.provider IS 'OAuth provider: google, facebook, microsoft, github, apple';
COMMENT ON COLUMN oauth_providers.access_token IS 'Encrypted OAuth access token from provider';

-- ============================================================================
-- 5. OAUTH_AUTHORIZATION_CODES TABLE (OAuth Flow Temporary Codes)
-- ============================================================================

CREATE TABLE oauth_authorization_codes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(255) NOT NULL UNIQUE,
    member_user_id UUID NOT NULL,
    client_app_id UUID NOT NULL,
    redirect_uri TEXT NOT NULL,
    scopes TEXT[] NULL,
    code_challenge VARCHAR(255) NULL,
    code_challenge_method VARCHAR(10) NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Foreign keys
    CONSTRAINT fk_oauth_auth_codes_user 
        FOREIGN KEY (member_user_id) 
        REFERENCES member_user(id) 
        ON DELETE CASCADE,
    
    CONSTRAINT fk_oauth_auth_codes_client 
        FOREIGN KEY (client_app_id) 
        REFERENCES client_app(id) 
        ON DELETE CASCADE,
    
    -- Check constraints
    CONSTRAINT chk_code_challenge_method 
        CHECK (code_challenge_method IS NULL OR code_challenge_method IN ('S256', 'plain'))
);

-- Indexes
CREATE INDEX idx_oauth_auth_codes_code 
    ON oauth_authorization_codes(code) 
    WHERE used_at IS NULL;

CREATE INDEX idx_oauth_auth_codes_expires 
    ON oauth_authorization_codes(expires_at);

CREATE INDEX idx_oauth_auth_codes_user 
    ON oauth_authorization_codes(member_user_id);

COMMENT ON TABLE oauth_authorization_codes IS 'Temporary OAuth 2.0 authorization codes (PKCE support)';
COMMENT ON COLUMN oauth_authorization_codes.code_challenge IS 'PKCE code challenge for enhanced security';
COMMENT ON COLUMN oauth_authorization_codes.code_challenge_method IS 'PKCE method: S256 (SHA256) or plain';

-- ============================================================================
-- 6. PASSWORD_RESET_TOKENS TABLE (Password Reset Management)
-- ============================================================================

CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_user_id UUID NOT NULL,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ NULL,
    ip_address INET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Foreign keys
    CONSTRAINT fk_password_reset_user 
        FOREIGN KEY (member_user_id) 
        REFERENCES member_user(id) 
        ON DELETE CASCADE
);

-- Indexes
CREATE INDEX idx_password_reset_hash 
    ON password_reset_tokens(token_hash) 
    WHERE used_at IS NULL;

CREATE INDEX idx_password_reset_expires 
    ON password_reset_tokens(expires_at);

CREATE INDEX idx_password_reset_user 
    ON password_reset_tokens(member_user_id);

COMMENT ON TABLE password_reset_tokens IS 'Password reset token management';
COMMENT ON COLUMN password_reset_tokens.token_hash IS 'SHA256 hash of the reset token';
COMMENT ON COLUMN password_reset_tokens.expires_at IS 'Token expires after 1 hour';

-- ============================================================================
-- 7. EMAIL_VERIFICATION_TOKENS TABLE (Email Verification Management)
-- ============================================================================

CREATE TABLE email_verification_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_user_id UUID NOT NULL,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    verified_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Foreign keys
    CONSTRAINT fk_email_verification_user 
        FOREIGN KEY (member_user_id) 
        REFERENCES member_user(id) 
        ON DELETE CASCADE
);

-- Indexes
CREATE INDEX idx_email_verification_hash 
    ON email_verification_tokens(token_hash) 
    WHERE verified_at IS NULL;

CREATE INDEX idx_email_verification_expires 
    ON email_verification_tokens(expires_at);

CREATE INDEX idx_email_verification_user 
    ON email_verification_tokens(member_user_id);

CREATE INDEX idx_email_verification_email 
    ON email_verification_tokens(email) 
    WHERE verified_at IS NULL;

COMMENT ON TABLE email_verification_tokens IS 'Email verification token management';
COMMENT ON COLUMN email_verification_tokens.expires_at IS 'Token expires after 24 hours';

-- ============================================================================
-- 8. TWO_FACTOR_AUTH TABLE (2FA/MFA Support)
-- ============================================================================

CREATE TABLE two_factor_auth (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    member_user_id UUID NOT NULL,
    method VARCHAR(20) NOT NULL,
    secret TEXT NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    backup_codes TEXT[] NULL,
    last_used TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modified_at TIMESTAMPTZ NULL,
    
    -- Foreign keys
    CONSTRAINT fk_two_factor_user 
        FOREIGN KEY (member_user_id) 
        REFERENCES member_user(id) 
        ON DELETE CASCADE,
    
    -- Check constraints
    CONSTRAINT chk_2fa_method CHECK (method IN ('totp', 'sms', 'email'))
);

-- Unique constraint: one 2FA method per user
CREATE UNIQUE INDEX idx_two_factor_user_method 
    ON two_factor_auth(member_user_id, method);

-- Performance indexes
CREATE INDEX idx_two_factor_enabled 
    ON two_factor_auth(member_user_id, is_enabled) 
    WHERE is_enabled = TRUE;

COMMENT ON TABLE two_factor_auth IS 'Two-factor authentication configuration';
COMMENT ON COLUMN two_factor_auth.method IS '2FA method: totp (authenticator app), sms, email';
COMMENT ON COLUMN two_factor_auth.secret IS 'Encrypted TOTP secret or SMS/email configuration';
COMMENT ON COLUMN two_factor_auth.backup_codes IS 'Encrypted one-time backup codes';

-- ============================================================================
-- 9. OAUTH_STATES TABLE (OAuth State Parameter Tracking)
-- ============================================================================

CREATE TABLE oauth_states (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    state VARCHAR(255) NOT NULL UNIQUE,
    provider VARCHAR(50) NOT NULL,
    redirect_uri TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Check constraints
    CONSTRAINT chk_oauth_state_provider CHECK (provider IN ('google', 'facebook', 'microsoft', 'github', 'apple'))
);

-- Indexes
CREATE INDEX idx_oauth_states_state 
    ON oauth_states(state) 
    WHERE used_at IS NULL;

CREATE INDEX idx_oauth_states_expires 
    ON oauth_states(expires_at);

COMMENT ON TABLE oauth_states IS 'OAuth state parameter tracking for CSRF protection';
COMMENT ON COLUMN oauth_states.state IS 'Random state value sent to OAuth provider';

-- ============================================================================
-- End of 002_authentication_tables.sql
-- ============================================================================