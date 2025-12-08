-- Insert default roles
INSERT INTO roles (name, description) VALUES
    ('admin', 'Administrator with user management permissions'),
    ('user', 'Regular user with weather and time access')
ON CONFLICT (name) DO NOTHING;

-- Insert default permissions
INSERT INTO permissions (name, resource, action) VALUES
    ('user:read', 'user', 'read'),
    ('user:write', 'user', 'write'),
    ('weather:read', 'weather', 'read'),
    ('time:read', 'time', 'read')
ON CONFLICT (name) DO NOTHING;

-- Assign permissions to admin role
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'admin' AND p.name IN ('user:read', 'user:write')
ON CONFLICT (role_id, permission_id) DO NOTHING;

-- Assign permissions to user role
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.name = 'user' AND p.name IN ('weather:read', 'time:read')
ON CONFLICT (role_id, permission_id) DO NOTHING;

