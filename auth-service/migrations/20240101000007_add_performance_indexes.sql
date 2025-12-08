-- Additional indexes for performance optimization

-- Index for user lookups by active status
CREATE INDEX IF NOT EXISTS idx_users_is_active ON users(is_active) WHERE is_active = true;

-- Composite index for user-role lookups
CREATE INDEX IF NOT EXISTS idx_user_roles_composite ON user_roles(user_id, role_id);

-- Composite index for role-permission lookups
CREATE INDEX IF NOT EXISTS idx_role_permissions_composite ON role_permissions(role_id, permission_id);

-- Index for users created_at for sorting
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at DESC);

