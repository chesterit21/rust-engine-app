-- ============================================================================
-- SSO Fullstack Rust - PostgreSQL Migration Script
-- File: 001_initial_schema.sql
-- Description: Core tables creation (Users, Client Apps, Menus, Groups, Tenants)
-- ============================================================================

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================================
-- 1. MEMBER_USER TABLE (Core User Accounts)
-- ============================================================================

CREATE TABLE member_user (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    display_name VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL,
    password VARCHAR(255) NULL, -- NULL for OAuth-only users
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_login BOOLEAN NOT NULL DEFAULT FALSE,
    last_login TIMESTAMPTZ NULL,
    status_member VARCHAR(50) NOT NULL DEFAULT 'new_register',
    link_profile_image TEXT NULL,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    phone_number VARCHAR(20) NULL,
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NULL,
    modified_at TIMESTAMPTZ NULL,
    modified_by UUID NULL,
    removed_at TIMESTAMPTZ NULL,
    removed_by UUID NULL,
    approved_at TIMESTAMPTZ NULL,
    approved_by UUID NULL,
    
    -- Constraints
    CONSTRAINT chk_status_member CHECK (status_member IN ('new_register', 'wait_validation', 'invitation', 'active', 'suspended', 'blocked'))
);

-- Create unique indexes for active records only
CREATE UNIQUE INDEX idx_member_user_email_unique 
    ON member_user(email) 
    WHERE removed_at IS NULL;

CREATE UNIQUE INDEX idx_member_user_display_name_unique 
    ON member_user(display_name) 
    WHERE removed_at IS NULL;

-- Performance indexes
CREATE INDEX idx_member_user_status 
    ON member_user(status_member) 
    WHERE removed_at IS NULL;

CREATE INDEX idx_member_user_active 
    ON member_user(is_active, is_login) 
    WHERE removed_at IS NULL;

COMMENT ON TABLE member_user IS 'Core user accounts table supporting both internal and OAuth authentication';
COMMENT ON COLUMN member_user.password IS 'Bcrypt hashed password. NULL for OAuth-only users';
COMMENT ON COLUMN member_user.status_member IS 'User status: new_register, wait_validation, invitation, active, suspended, blocked';

-- ============================================================================
-- 2. CLIENT_APP TABLE (Registered Applications)
-- ============================================================================

CREATE TABLE client_app (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    description TEXT NULL,
    unique_name VARCHAR(100) NOT NULL,
    type_app VARCHAR(50) NOT NULL,
    url_app TEXT NOT NULL,
    client_secret VARCHAR(255) NOT NULL, -- Hashed secret
    redirect_uris TEXT[] NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NULL,
    modified_at TIMESTAMPTZ NULL,
    modified_by UUID NULL,
    removed_at TIMESTAMPTZ NULL,
    removed_by UUID NULL,
    approved_at TIMESTAMPTZ NULL,
    approved_by UUID NULL,
    
    -- Constraints
    CONSTRAINT chk_type_app CHECK (type_app IN ('web', 'mobile', 'desktop', 'api', 'console'))
);

-- Unique constraint on unique_name for active records
CREATE UNIQUE INDEX idx_client_app_unique_name 
    ON client_app(unique_name) 
    WHERE removed_at IS NULL;

CREATE UNIQUE INDEX idx_client_app_name 
    ON client_app(name) 
    WHERE removed_at IS NULL;

CREATE INDEX idx_client_app_type 
    ON client_app(type_app) 
    WHERE removed_at IS NULL;

COMMENT ON TABLE client_app IS 'Registered client applications that integrate with SSO';
COMMENT ON COLUMN client_app.unique_name IS 'URL-friendly unique identifier (slug)';
COMMENT ON COLUMN client_app.type_app IS 'Application type: web, mobile, desktop, api, console';
COMMENT ON COLUMN client_app.client_secret IS 'OAuth client secret (hashed with bcrypt)';

-- ============================================================================
-- 3. MENU_APP TABLE (Application Menus for RBAC)
-- ============================================================================

CREATE TABLE menu_app (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    client_app_id UUID NOT NULL,
    menu_name VARCHAR(100) NOT NULL,
    menu_url VARCHAR(255) NOT NULL,
    parent_menu_id UUID NULL,
    menu_icon VARCHAR(100) NULL,
    menu_level INTEGER NOT NULL DEFAULT 1,
    menu_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NULL,
    modified_at TIMESTAMPTZ NULL,
    modified_by UUID NULL,
    removed_at TIMESTAMPTZ NULL,
    removed_by UUID NULL,
    approved_at TIMESTAMPTZ NULL,
    approved_by UUID NULL,
    
    -- Foreign keys
    CONSTRAINT fk_menu_app_client 
        FOREIGN KEY (client_app_id) 
        REFERENCES client_app(id) 
        ON DELETE CASCADE,
    
    CONSTRAINT fk_menu_app_parent 
        FOREIGN KEY (parent_menu_id) 
        REFERENCES menu_app(id) 
        ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_menu_level CHECK (menu_level > 0 AND menu_level <= 5)
);

-- Unique constraint: menu_name must be unique per client_app
CREATE UNIQUE INDEX idx_menu_app_name_per_client 
    ON menu_app(client_app_id, menu_name) 
    WHERE removed_at IS NULL;

-- Performance indexes
CREATE INDEX idx_menu_app_client 
    ON menu_app(client_app_id) 
    WHERE removed_at IS NULL;

CREATE INDEX idx_menu_app_parent 
    ON menu_app(parent_menu_id) 
    WHERE removed_at IS NULL;

CREATE INDEX idx_menu_app_level_order 
    ON menu_app(client_app_id, menu_level, menu_order) 
    WHERE removed_at IS NULL AND is_active = TRUE;

COMMENT ON TABLE menu_app IS 'Application menu items for role-based access control';
COMMENT ON COLUMN menu_app.menu_level IS 'Menu hierarchy depth (1=root, max 5 levels)';
COMMENT ON COLUMN menu_app.menu_order IS 'Display order within same level';

-- ============================================================================
-- 4. GROUP_APP TABLE (User Groups/Roles)
-- ============================================================================

CREATE TABLE group_app (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    client_app_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NULL,
    modified_at TIMESTAMPTZ NULL,
    modified_by UUID NULL,
    removed_at TIMESTAMPTZ NULL,
    removed_by UUID NULL,
    approved_at TIMESTAMPTZ NULL,
    approved_by UUID NULL,
    
    -- Foreign keys
    CONSTRAINT fk_group_app_client 
        FOREIGN KEY (client_app_id) 
        REFERENCES client_app(id) 
        ON DELETE CASCADE
);

-- Unique constraint: group name must be unique per client_app
CREATE UNIQUE INDEX idx_group_app_name_per_client 
    ON group_app(client_app_id, name) 
    WHERE removed_at IS NULL;

CREATE INDEX idx_group_app_client 
    ON group_app(client_app_id) 
    WHERE removed_at IS NULL;

COMMENT ON TABLE group_app IS 'User groups/roles within client applications';
COMMENT ON COLUMN group_app.name IS 'Group name (e.g., Admin, User, Manager)';

-- ============================================================================
-- 5. GROUP_MENU_APP TABLE (Permission Matrix)
-- ============================================================================

CREATE TABLE group_menu_app (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    client_app_id UUID NOT NULL,
    group_app_id UUID NOT NULL,
    menu_app_id UUID NOT NULL,
    
    -- Permission flags
    is_view BOOLEAN NOT NULL DEFAULT FALSE,
    is_add BOOLEAN NOT NULL DEFAULT FALSE,
    is_edit BOOLEAN NOT NULL DEFAULT FALSE,
    is_delete BOOLEAN NOT NULL DEFAULT FALSE,
    is_approve BOOLEAN NOT NULL DEFAULT FALSE,
    is_download BOOLEAN NOT NULL DEFAULT FALSE,
    is_upload BOOLEAN NOT NULL DEFAULT FALSE,
    is_print BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NULL,
    modified_at TIMESTAMPTZ NULL,
    modified_by UUID NULL,
    removed_at TIMESTAMPTZ NULL,
    removed_by UUID NULL,
    approved_at TIMESTAMPTZ NULL,
    approved_by UUID NULL,
    
    -- Foreign keys
    CONSTRAINT fk_group_menu_app_client 
        FOREIGN KEY (client_app_id) 
        REFERENCES client_app(id) 
        ON DELETE CASCADE,
    
    CONSTRAINT fk_group_menu_app_group 
        FOREIGN KEY (group_app_id) 
        REFERENCES group_app(id) 
        ON DELETE CASCADE,
    
    CONSTRAINT fk_group_menu_app_menu 
        FOREIGN KEY (menu_app_id) 
        REFERENCES menu_app(id) 
        ON DELETE CASCADE
);

-- Unique constraint: one permission set per group-menu combination
CREATE UNIQUE INDEX idx_group_menu_app_unique 
    ON group_menu_app(client_app_id, group_app_id, menu_app_id) 
    WHERE removed_at IS NULL;

-- Performance indexes
CREATE INDEX idx_group_menu_app_group 
    ON group_menu_app(group_app_id) 
    WHERE removed_at IS NULL;

CREATE INDEX idx_group_menu_app_menu 
    ON group_menu_app(menu_app_id) 
    WHERE removed_at IS NULL;

COMMENT ON TABLE group_menu_app IS 'Permission matrix defining what each group can do on each menu';
COMMENT ON COLUMN group_menu_app.is_view IS 'Permission to view/read';
COMMENT ON COLUMN group_menu_app.is_add IS 'Permission to create new records';
COMMENT ON COLUMN group_menu_app.is_edit IS 'Permission to update existing records';

-- ============================================================================
-- 6. USER_GROUP_APP TABLE (User-Group Assignments)
-- ============================================================================

CREATE TABLE user_group_app (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    client_app_id UUID NOT NULL,
    member_user_id UUID NOT NULL,
    group_app_id UUID NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NULL,
    modified_at TIMESTAMPTZ NULL,
    modified_by UUID NULL,
    removed_at TIMESTAMPTZ NULL,
    removed_by UUID NULL,
    approved_at TIMESTAMPTZ NULL,
    approved_by UUID NULL,
    
    -- Foreign keys
    CONSTRAINT fk_user_group_app_client 
        FOREIGN KEY (client_app_id) 
        REFERENCES client_app(id) 
        ON DELETE CASCADE,
    
    CONSTRAINT fk_user_group_app_user 
        FOREIGN KEY (member_user_id) 
        REFERENCES member_user(id) 
        ON DELETE CASCADE,
    
    CONSTRAINT fk_user_group_app_group 
        FOREIGN KEY (group_app_id) 
        REFERENCES group_app(id) 
        ON DELETE CASCADE
);

-- Unique constraint: user can only be in a group once per client app
CREATE UNIQUE INDEX idx_user_group_app_unique 
    ON user_group_app(client_app_id, member_user_id, group_app_id) 
    WHERE removed_at IS NULL;

-- Performance indexes
CREATE INDEX idx_user_group_app_user 
    ON user_group_app(member_user_id) 
    WHERE removed_at IS NULL AND is_active = TRUE;

CREATE INDEX idx_user_group_app_group 
    ON user_group_app(group_app_id) 
    WHERE removed_at IS NULL AND is_active = TRUE;

COMMENT ON TABLE user_group_app IS 'Links users to groups - users can belong to multiple groups';

-- ============================================================================
-- 7. TENANT_APP TABLE (Multi-Tenancy Support)
-- ============================================================================

CREATE TABLE tenant_app (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    description TEXT NULL,
    slug VARCHAR(100) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    max_users INTEGER NOT NULL DEFAULT 10,
    subscription_plan VARCHAR(50) NOT NULL DEFAULT 'free',
    subscription_expires_at TIMESTAMPTZ NULL,
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NULL,
    modified_at TIMESTAMPTZ NULL,
    modified_by UUID NULL,
    removed_at TIMESTAMPTZ NULL,
    removed_by UUID NULL,
    approved_at TIMESTAMPTZ NULL,
    approved_by UUID NULL,
    
    -- Constraints
    CONSTRAINT chk_subscription_plan CHECK (subscription_plan IN ('free', 'basic', 'premium', 'enterprise')),
    CONSTRAINT chk_max_users CHECK (max_users > 0)
);

-- Unique constraints for active tenants
CREATE UNIQUE INDEX idx_tenant_app_name 
    ON tenant_app(name) 
    WHERE removed_at IS NULL;

CREATE UNIQUE INDEX idx_tenant_app_slug 
    ON tenant_app(slug) 
    WHERE removed_at IS NULL;

CREATE INDEX idx_tenant_app_active 
    ON tenant_app(is_active, subscription_plan) 
    WHERE removed_at IS NULL;

COMMENT ON TABLE tenant_app IS 'Multi-tenant organizations/workspaces';
COMMENT ON COLUMN tenant_app.slug IS 'URL-friendly unique identifier';
COMMENT ON COLUMN tenant_app.subscription_plan IS 'Subscription tier: free, basic, premium, enterprise';

-- ============================================================================
-- 8. MEMBER_USER_TENANT_APP TABLE (User-Tenant Relationships)
-- ============================================================================

CREATE TABLE member_user_tenant_app (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_app_id UUID NOT NULL,
    member_user_id UUID NOT NULL,
    is_owner BOOLEAN NOT NULL DEFAULT FALSE,
    level_owner INTEGER NULL,
    role_in_tenant VARCHAR(50) NOT NULL DEFAULT 'member',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NULL,
    modified_at TIMESTAMPTZ NULL,
    modified_by UUID NULL,
    removed_at TIMESTAMPTZ NULL,
    removed_by UUID NULL,
    approved_at TIMESTAMPTZ NULL,
    approved_by UUID NULL,
    
    -- Foreign keys
    CONSTRAINT fk_member_user_tenant_tenant 
        FOREIGN KEY (tenant_app_id) 
        REFERENCES tenant_app(id) 
        ON DELETE CASCADE,
    
    CONSTRAINT fk_member_user_tenant_user 
        FOREIGN KEY (member_user_id) 
        REFERENCES member_user(id) 
        ON DELETE CASCADE,
    
    -- Check constraints
    CONSTRAINT chk_role_in_tenant CHECK (role_in_tenant IN ('owner', 'admin', 'member', 'guest')),
    CONSTRAINT chk_level_owner CHECK (
        (is_owner = FALSE AND level_owner IS NULL) OR 
        (is_owner = TRUE AND level_owner > 0)
    )
);

-- Unique constraint: user can only belong to a tenant once
CREATE UNIQUE INDEX idx_member_user_tenant_unique 
    ON member_user_tenant_app(tenant_app_id, member_user_id) 
    WHERE removed_at IS NULL;

-- Unique constraint: only one owner per level per tenant
CREATE UNIQUE INDEX idx_member_user_tenant_level_owner 
    ON member_user_tenant_app(tenant_app_id, level_owner) 
    WHERE removed_at IS NULL AND is_owner = TRUE;

-- Performance indexes
CREATE INDEX idx_member_user_tenant_user 
    ON member_user_tenant_app(member_user_id) 
    WHERE removed_at IS NULL;

CREATE INDEX idx_member_user_tenant_tenant 
    ON member_user_tenant_app(tenant_app_id) 
    WHERE removed_at IS NULL;

CREATE INDEX idx_member_user_tenant_owners 
    ON member_user_tenant_app(tenant_app_id, is_owner, level_owner) 
    WHERE removed_at IS NULL AND is_owner = TRUE;

COMMENT ON TABLE member_user_tenant_app IS 'User-tenant relationships with ownership levels';
COMMENT ON COLUMN member_user_tenant_app.level_owner IS 'Owner hierarchy (1=primary, 2=secondary, etc.)';
COMMENT ON COLUMN member_user_tenant_app.role_in_tenant IS 'User role: owner, admin, member, guest';

-- ============================================================================
-- 9. TENANT_TRANSACTION_APP TABLE (Tenant Ownership Transfers)
-- ============================================================================

CREATE TABLE tenant_transaction_app (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_app_id UUID NOT NULL,
    from_member_user_id UUID NOT NULL,
    to_member_user_id UUID NOT NULL,
    transaction_type VARCHAR(50) NOT NULL,
    transaction_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    notes TEXT NULL,
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NULL,
    modified_at TIMESTAMPTZ NULL,
    modified_by UUID NULL,
    removed_at TIMESTAMPTZ NULL,
    removed_by UUID NULL,
    approved_at TIMESTAMPTZ NULL,
    approved_by UUID NULL,
    
    -- Foreign keys
    CONSTRAINT fk_tenant_transaction_tenant 
        FOREIGN KEY (tenant_app_id) 
        REFERENCES tenant_app(id) 
        ON DELETE CASCADE,
    
    CONSTRAINT fk_tenant_transaction_from 
        FOREIGN KEY (from_member_user_id) 
        REFERENCES member_user(id) 
        ON DELETE RESTRICT,
    
    CONSTRAINT fk_tenant_transaction_to 
        FOREIGN KEY (to_member_user_id) 
        REFERENCES member_user(id) 
        ON DELETE RESTRICT,
    
    -- Check constraints
    CONSTRAINT chk_transaction_type CHECK (transaction_type IN ('transfer', 'invitation', 'promotion', 'demotion')),
    CONSTRAINT chk_different_users CHECK (from_member_user_id != to_member_user_id)
);

-- Performance indexes
CREATE INDEX idx_tenant_transaction_tenant 
    ON tenant_transaction_app(tenant_app_id, transaction_date DESC);

CREATE INDEX idx_tenant_transaction_from 
    ON tenant_transaction_app(from_member_user_id);

CREATE INDEX idx_tenant_transaction_to 
    ON tenant_transaction_app(to_member_user_id);

COMMENT ON TABLE tenant_transaction_app IS 'Audit trail of tenant ownership changes';
COMMENT ON COLUMN tenant_transaction_app.transaction_type IS 'Type: transfer, invitation, promotion, demotion';

-- ============================================================================
-- End of 001_initial_schema.sql
-- ============================================================================