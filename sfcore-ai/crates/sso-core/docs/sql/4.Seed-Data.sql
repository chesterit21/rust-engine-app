-- ============================================================================
-- SSO Fullstack Rust - PostgreSQL Seed Data Script
-- File: 004_seed_data.sql
-- Description: Dummy data for development and testing
-- ============================================================================

-- ============================================================================
-- IMPORTANT NOTES:
-- 1. All passwords are hashed with bcrypt (cost factor 12)
-- 2. Default password for all users: "Password123!"
-- 3. Bcrypt hash: $2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzNLmVF3Tm
-- 4. Super Admin email: admin@sso-system.com
-- 5. This is for DEVELOPMENT/TESTING only - DO NOT use in production
-- ============================================================================

BEGIN;

-- ============================================================================
-- 1. MEMBER_USER - Create Users
-- ============================================================================

-- Super Admin (Internal SSO System Admin)
INSERT INTO member_user (
    id, 
    display_name, 
    email, 
    password, 
    is_active, 
    status_member, 
    email_verified,
    created_at,
    approved_at
) VALUES 
(
    '00000000-0000-0000-0000-000000000001'::UUID,
    'System Administrator',
    'admin@sso-system.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzNLmVF3Tm', -- Password123!
    TRUE,
    'active',
    TRUE,
    NOW(),
    NOW()
),
-- Tenant Owner 1
(
    '00000000-0000-0000-0000-000000000002'::UUID,
    'John Doe',
    'john.doe@acme-corp.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzNLmVF3Tm',
    TRUE,
    'active',
    TRUE,
    NOW(),
    NOW()
),
-- Tenant Owner 2
(
    '00000000-0000-0000-0000-000000000003'::UUID,
    'Jane Smith',
    'jane.smith@acme-corp.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzNLmVF3Tm',
    TRUE,
    'active',
    TRUE,
    NOW(),
    NOW()
),
-- Regular User 1
(
    '00000000-0000-0000-0000-000000000004'::UUID,
    'Alice Johnson',
    'alice.johnson@acme-corp.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzNLmVF3Tm',
    TRUE,
    'active',
    TRUE,
    NOW(),
    NOW()
),
-- Regular User 2
(
    '00000000-0000-0000-0000-000000000005'::UUID,
    'Bob Williams',
    'bob.williams@acme-corp.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzNLmVF3Tm',
    TRUE,
    'active',
    TRUE,
    NOW(),
    NOW()
),
-- Invited User (not yet accepted)
(
    '00000000-0000-0000-0000-000000000006'::UUID,
    'Carol Davis',
    'carol.davis@acme-corp.com',
    NULL, -- No password yet (invitation pending)
    TRUE,
    'invitation',
    FALSE,
    NOW(),
    NULL
),
-- OAuth User (Google)
(
    '00000000-0000-0000-0000-000000000007'::UUID,
    'David Brown',
    'david.brown@gmail.com',
    NULL, -- OAuth user, no password
    TRUE,
    'active',
    TRUE,
    NOW(),
    NOW()
);

-- ============================================================================
-- 2. TENANT_APP - Create Tenants
-- ============================================================================

INSERT INTO tenant_app (
    id,
    name,
    description,
    slug,
    is_active,
    max_users,
    subscription_plan,
    subscription_expires_at,
    created_at,
    created_by,
    approved_at
) VALUES 
(
    '10000000-0000-0000-0000-000000000001'::UUID,
    'ACME Corporation',
    'Main corporate tenant for ACME Corporation',
    'acme-corp',
    TRUE,
    50,
    'enterprise',
    NOW() + INTERVAL '1 year',
    NOW(),
    '00000000-0000-0000-0000-000000000002'::UUID,
    NOW()
),
(
    '10000000-0000-0000-0000-000000000002'::UUID,
    'Tech Startup Inc',
    'Technology startup company tenant',
    'tech-startup',
    TRUE,
    20,
    'premium',
    NOW() + INTERVAL '6 months',
    NOW(),
    '00000000-0000-0000-0000-000000000003'::UUID,
    NOW()
);

-- ============================================================================
-- 3. MEMBER_USER_TENANT_APP - Link Users to Tenants
-- ============================================================================

INSERT INTO member_user_tenant_app (
    id,
    tenant_app_id,
    member_user_id,
    is_owner,
    level_owner,
    role_in_tenant,
    joined_at,
    created_at
) VALUES 
-- ACME Corp - Primary Owner
(
    '20000000-0000-0000-0000-000000000001'::UUID,
    '10000000-0000-0000-0000-000000000001'::UUID,
    '00000000-0000-0000-0000-000000000002'::UUID,
    TRUE,
    1,
    'owner',
    NOW(),
    NOW()
),
-- ACME Corp - Secondary Owner
(
    '20000000-0000-0000-0000-000000000002'::UUID,
    '10000000-0000-0000-0000-000000000001'::UUID,
    '00000000-0000-0000-0000-000000000003'::UUID,
    TRUE,
    2,
    'owner',
    NOW(),
    NOW()
),
-- ACME Corp - Admin
(
    '20000000-0000-0000-0000-000000000003'::UUID,
    '10000000-0000-0000-0000-000000000001'::UUID,
    '00000000-0000-0000-0000-000000000004'::UUID,
    FALSE,
    NULL,
    'admin',
    NOW(),
    NOW()
),
-- ACME Corp - Regular Member
(
    '20000000-0000-0000-0000-000000000004'::UUID,
    '10000000-0000-0000-0000-000000000001'::UUID,
    '00000000-0000-0000-0000-000000000005'::UUID,
    FALSE,
    NULL,
    'member',
    NOW(),
    NOW()
),
-- Tech Startup - Owner
(
    '20000000-0000-0000-0000-000000000005'::UUID,
    '10000000-0000-0000-0000-000000000002'::UUID,
    '00000000-0000-0000-0000-000000000003'::UUID,
    TRUE,
    1,
    'owner',
    NOW(),
    NOW()
);

-- ============================================================================
-- 4. CLIENT_APP - Create Client Applications
-- ============================================================================

INSERT INTO client_app (
    id,
    name,
    description,
    unique_name,
    type_app,
    url_app,
    client_secret,
    redirect_uris,
    is_active,
    created_at,
    created_by,
    approved_at
) VALUES 
(
    '30000000-0000-0000-0000-000000000001'::UUID,
    'ACME Dashboard',
    'Main dashboard application for ACME Corporation',
    'acme-dashboard',
    'web',
    'https://dashboard.acme-corp.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzNLmVF3Tm', -- Hashed secret
    ARRAY['https://dashboard.acme-corp.com/callback', 'http://localhost:3000/callback'],
    TRUE,
    NOW(),
    '00000000-0000-0000-0000-000000000002'::UUID,
    NOW()
),
(
    '30000000-0000-0000-0000-000000000002'::UUID,
    'ACME Mobile App',
    'Mobile application for ACME Corporation',
    'acme-mobile',
    'mobile',
    'com.acme.app://callback',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzNLmVF3Tm',
    ARRAY['com.acme.app://callback'],
    TRUE,
    NOW(),
    '00000000-0000-0000-0000-000000000002'::UUID,
    NOW()
),
(
    '30000000-0000-0000-0000-000000000003'::UUID,
    'ACME API Gateway',
    'RESTful API for ACME services',
    'acme-api',
    'api',
    'https://api.acme-corp.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzNLmVF3Tm',
    ARRAY['https://api.acme-corp.com/oauth/callback'],
    TRUE,
    NOW(),
    '00000000-0000-0000-0000-000000000002'::UUID,
    NOW()
);

-- ============================================================================
-- 5. MENU_APP - Create Application Menus
-- ============================================================================

-- Dashboard Menus
INSERT INTO menu_app (
    id,
    client_app_id,
    menu_name,
    menu_url,
    parent_menu_id,
    menu_icon,
    menu_level,
    menu_order,
    is_active,
    created_at
) VALUES 
-- Root Level Menus
(
    '40000000-0000-0000-0000-000000000001'::UUID,
    '30000000-0000-0000-0000-000000000001'::UUID,
    'Dashboard',
    '/dashboard',
    NULL,
    'fa-home',
    1,
    1,
    TRUE,
    NOW()
),
(
    '40000000-0000-0000-0000-000000000002'::UUID,
    '30000000-0000-0000-0000-000000000001'::UUID,
    'Users',
    '/users',
    NULL,
    'fa-users',
    1,
    2,
    TRUE,
    NOW()
),
(
    '40000000-0000-0000-0000-000000000003'::UUID,
    '30000000-0000-0000-0000-000000000001'::UUID,
    'Settings',
    '/settings',
    NULL,
    'fa-cog',
    1,
    3,
    TRUE,
    NOW()
),
(
    '40000000-0000-0000-0000-000000000004'::UUID,
    '30000000-0000-0000-0000-000000000001'::UUID,
    'Reports',
    '/reports',
    NULL,
    'fa-chart-bar',
    1,
    4,
    TRUE,
    NOW()
),
-- Sub Menus under Users
(
    '40000000-0000-0000-0000-000000000005'::UUID,
    '30000000-0000-0000-0000-000000000001'::UUID,
    'User List',
    '/users/list',
    '40000000-0000-0000-0000-000000000002'::UUID,
    'fa-list',
    2,
    1,
    TRUE,
    NOW()
),
(
    '40000000-0000-0000-0000-000000000006'::UUID,
    '30000000-0000-0000-0000-000000000001'::UUID,
    'Add User',
    '/users/add',
    '40000000-0000-0000-0000-000000000002'::UUID,
    'fa-user-plus',
    2,
    2,
    TRUE,
    NOW()
),
(
    '40000000-0000-0000-0000-000000000007'::UUID,
    '30000000-0000-0000-0000-000000000001'::UUID,
    'Roles & Permissions',
    '/users/roles',
    '40000000-0000-0000-0000-000000000002'::UUID,
    'fa-shield-alt',
    2,
    3,
    TRUE,
    NOW()
),
-- Sub Menus under Settings
(
    '40000000-0000-0000-0000-000000000008'::UUID,
    '30000000-0000-0000-0000-000000000001'::UUID,
    'General Settings',
    '/settings/general',
    '40000000-0000-0000-0000-000000000003'::UUID,
    'fa-sliders-h',
    2,
    1,
    TRUE,
    NOW()
),
(
    '40000000-0000-0000-0000-000000000009'::UUID,
    '30000000-0000-0000-0000-000000000001'::UUID,
    'Security',
    '/settings/security',
    '40000000-0000-0000-0000-000000000003'::UUID,
    'fa-lock',
    2,
    2,
    TRUE,
    NOW()
),
(
    '40000000-0000-0000-0000-000000000010'::UUID,
    '30000000-0000-0000-0000-000000000001'::UUID,
    'Integrations',
    '/settings/integrations',
    '40000000-0000-0000-0000-000000000003'::UUID,
    'fa-plug',
    2,
    3,
    TRUE,
    NOW()
);

-- ============================================================================
-- 6. GROUP_APP - Create User Groups/Roles
-- ============================================================================

INSERT INTO group_app (
    id,
    client_app_id,
    name,
    description,
    is_active,
    created_at,
    approved_at
) VALUES 
-- ACME Dashboard Groups
(
    '50000000-0000-0000-0000-000000000001'::UUID,
    '30000000-0000-0000-0000-000000000001'::UUID,
    'Super Admin',
    'Full system access with all permissions',
    TRUE,
    NOW(),
    NOW()
),
(
    '50000000-0000-0000-0000-000000000002'::UUID,
    '30000000-0000-0000-0000-000000000001'::UUID,
    'Administrator',
    'Administrative access with most permissions',
    TRUE,
    NOW(),
    NOW()
),
(
    '50000000-0000-0000-0000-000000000003'::UUID,
    '30000000-0000-0000-0000-000000000001'::UUID,
    'Manager',
    'Management level access',
    TRUE,
    NOW(),
    NOW()
),
(
    '50000000-0000-0000-0000-000000000004'::UUID,
    '30000000-0000-0000-0000-000000000001'::UUID,
    'User',
    'Standard user access',
    TRUE,
    NOW(),
    NOW()
),
(
    '50000000-0000-0000-0000-000000000005'::UUID,
    '30000000-0000-0000-0000-000000000001'::UUID,
    'Read Only',
    'View-only access',
    TRUE,
    NOW(),
    NOW()
);

-- ============================================================================
-- 7. GROUP_MENU_APP - Assign Permissions to Groups
-- ============================================================================

-- Super Admin - Full Access to All Menus
INSERT INTO group_menu_app (
    client_app_id,
    group_app_id,
    menu_app_id,
    is_view,
    is_add,
    is_edit,
    is_delete,
    is_approve,
    is_download,
    is_upload,
    is_print,
    created_at
)
SELECT 
    '30000000-0000-0000-0000-000000000001'::UUID,
    '50000000-0000-0000-0000-000000000001'::UUID,
    id,
    TRUE,
    TRUE,
    TRUE,
    TRUE,
    TRUE,
    TRUE,
    TRUE,
    TRUE,
    NOW()
FROM menu_app 
WHERE client_app_id = '30000000-0000-0000-0000-000000000001'::UUID;

-- Administrator - Most Permissions (No Delete on Critical Menus)
INSERT INTO group_menu_app (
    client_app_id,
    group_app_id,
    menu_app_id,
    is_view,
    is_add,
    is_edit,
    is_delete,
    is_approve,
    is_download,
    is_upload,
    is_print,
    created_at
) VALUES 
-- Dashboard - Full Access
(
    '30000000-0000-0000-0000-000000000001'::UUID,
    '50000000-0000-0000-0000-000000000002'::UUID,
    '40000000-0000-0000-0000-000000000001'::UUID,
    TRUE, TRUE, TRUE, TRUE, TRUE, TRUE, TRUE, TRUE, NOW()
),
-- Users - No Delete
(
    '30000000-0000-0000-0000-000000000001'::UUID,
    '50000000-0000-0000-0000-000000000002'::UUID,
    '40000000-0000-0000-0000-000000000002'::UUID,
    TRUE, TRUE, TRUE, FALSE, TRUE, TRUE, TRUE, TRUE, NOW()
),
-- Settings - Full Access
(
    '30000000-0000-0000-0000-000000000001'::UUID,
    '50000000-0000-0000-0000-000000000002'::UUID,
    '40000000-0000-0000-0000-000000000003'::UUID,
    TRUE, TRUE, TRUE, TRUE, TRUE, TRUE, TRUE, TRUE, NOW()
),
-- Reports - View, Download, Print
(
    '30000000-0000-0000-0000-000000000001'::UUID,
    '50000000-0000-0000-0000-000000000002'::UUID,
    '40000000-0000-0000-0000-000000000004'::UUID,
    TRUE, FALSE, FALSE, FALSE, FALSE, TRUE, FALSE, TRUE, NOW()
);

-- Manager - Limited Permissions
INSERT INTO group_menu_app (
    client_app_id,
    group_app_id,
    menu_app_id,
    is_view,
    is_add,
    is_edit,
    is_delete,
    is_approve,
    is_download,
    is_upload,
    is_print,
    created_at
) VALUES 
(
    '30000000-0000-0000-0000-000000000001'::UUID,
    '50000000-0000-0000-0000-000000000003'::UUID,
    '40000000-0000-0000-0000-000000000001'::UUID,
    TRUE, FALSE, FALSE, FALSE, FALSE, TRUE, FALSE, TRUE, NOW()
),
(
    '30000000-0000-0000-0000-000000000001'::UUID,
    '50000000-0000-0000-0000-000000000003'::UUID,
    '40000000-0000-0000-0000-000000000002'::UUID,
    TRUE, TRUE, TRUE, FALSE, TRUE, TRUE, TRUE, TRUE, NOW()
);

-- Read Only - View & Download Only
INSERT INTO group_menu_app (
    client_app_id,
    group_app_id,
    menu_app_id,
    is_view,
    is_add,
    is_edit,
    is_delete,
    is_approve,
    is_download,
    is_upload,
    is_print,
    created_at
)
SELECT 
    '30000000-0000-0000-0000-000000000001'::UUID,
    '50000000-0000-0000-0000-000000000005'::UUID,
    id,
    TRUE,
    FALSE,
    FALSE,
    FALSE,
    FALSE,
    TRUE,
    FALSE,
    TRUE,
    NOW()
FROM menu_app 
WHERE client_app_id = '30000000-0000-0000-0000-000000000001'::UUID;

-- ============================================================================
-- 8. USER_GROUP_APP - Assign Users to Groups
-- ============================================================================

INSERT INTO user_group_app (
    client_app_id,
    member_user_id,
    group_app_id,
    is_active,
    created_at
) VALUES 
-- John Doe - Super Admin
(
    '30000000-0000-0000-0000-000000000001'::UUID,
    '00000000-0000-0000-0000-000000000002'::UUID,
    '50000000-0000-0000-0000-000000000001'::UUID,
    TRUE,
    NOW()
),
-- Jane Smith - Administrator
(
    '30000000-0000-0000-0000-000000000001'::UUID,
    '00000000-0000-0000-0000-000000000003'::UUID,
    '50000000-0000-0000-0000-000000000002'::UUID,
    TRUE,
    NOW()
),
-- Alice Johnson - Manager
(
    '30000000-0000-0000-0000-000000000001'::UUID,
    '00000000-0000-0000-0000-000000000004'::UUID,
    '50000000-0000-0000-0000-000000000003'::UUID,
    TRUE,
    NOW()
),
-- Bob Williams - User
(
    '30000000-0000-0000-0000-000000000001'::UUID,
    '00000000-0000-0000-0000-000000000005'::UUID,
    '50000000-0000-0000-0000-000000000004'::UUID,
    TRUE,
    NOW()
);

-- ============================================================================
-- 9. OAUTH_PROVIDERS - Link OAuth Accounts
-- ============================================================================

INSERT INTO oauth_providers (
    member_user_id,
    provider,
    provider_user_id,
    provider_email,
    provider_name,
    provider_avatar,
    created_at
) VALUES 
(
    '00000000-0000-0000-0000-000000000007'::UUID,
    'google',
    '1234567890',
    'david.brown@gmail.com',
    'David Brown',
    'https://lh3.googleusercontent.com/a/default-user',
    NOW()
);

-- ============================================================================
-- 10. TENANT_TRANSACTION_APP - Sample Tenant Transactions
-- ============================================================================

INSERT INTO tenant_transaction_app (
    tenant_app_id,
    from_member_user_id,
    to_member_user_id,
    transaction_type,
    transaction_date,
    notes,
    created_at
) VALUES 
(
    '10000000-0000-0000-0000-000000000001'::UUID,
    '00000000-0000-0000-0000-000000000002'::UUID,
    '00000000-0000-0000-0000-000000000004'::UUID,
    'promotion',
    NOW() - INTERVAL '30 days',
    'Promoted to admin role',
    NOW() - INTERVAL '30 days'
);

-- ============================================================================
-- 11. AUDIT_LOGS - Sample Audit Entries
-- ============================================================================

INSERT INTO audit_logs (
    member_user_id,
    action,
    entity_type,
    entity_id,
    new_values,
    ip_address,
    created_at
) VALUES 
(
    '00000000-0000-0000-0000-000000000002'::UUID,
    'login',
    'member_user',
    '00000000-0000-0000-0000-000000000002'::UUID,
    '{"success": true}'::jsonb,
    '192.168.1.100'::inet,
    NOW()
),
(
    '00000000-0000-0000-0000-000000000002'::UUID,
    'create',
    'client_app',
    '30000000-0000-0000-0000-000000000001'::UUID,
    '{"name": "ACME Dashboard"}'::jsonb,
    '192.168.1.100'::inet,
    NOW() - INTERVAL '1 day'
);

COMMIT;

-- ============================================================================
-- VERIFICATION QUERIES
-- ============================================================================

-- Verify data insertion
DO $$
BEGIN
    RAISE NOTICE 'Seed data insertion completed!';
    RAISE NOTICE 'Total users: %', (SELECT COUNT(*) FROM member_user);
    RAISE NOTICE 'Total tenants: %', (SELECT COUNT(*) FROM tenant_app);
    RAISE NOTICE 'Total client apps: %', (SELECT COUNT(*) FROM client_app);
    RAISE NOTICE 'Total menus: %', (SELECT COUNT(*) FROM menu_app);
    RAISE NOTICE 'Total groups: %', (SELECT COUNT(*) FROM group_app);
    RAISE NOTICE 'Total permissions: %', (SELECT COUNT(*) FROM group_menu_app);
END $$;

-- ============================================================================
-- USEFUL QUERIES FOR TESTING
-- ============================================================================

-- Get all permissions for Super Admin group
-- SELECT 
--     m.menu_name,
--     gm.is_view,
--     gm.is_add,
--     gm.is_edit,
--     gm.is_delete,
--     gm.is_approve
-- FROM group_menu_app gm
-- JOIN menu_app m ON gm.menu_app_id = m.id
-- WHERE gm.group_app_id = '50000000-0000-0000-0000-000000000001'::UUID
-- ORDER BY m.menu_order;

-- Get all groups for a user
-- SELECT 
--     u.display_name,
--     g.name as group_name,
--     c.name as client_app_name
-- FROM user_group_app ug
-- JOIN member_user u ON ug.member_user_id = u.id
-- JOIN group_app g ON ug.group_app_id = g.id
-- JOIN client_app c ON ug.client_app_id = c.id
-- WHERE u.email = 'john.doe@acme-corp.com';

-- ============================================================================
-- End of 004_seed_data.sql
-- ============================================================================