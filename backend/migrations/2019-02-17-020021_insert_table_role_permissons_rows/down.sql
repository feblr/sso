DELETE FROM sso.role_permissions
WHERE sso.role_permissions.role_id IN (SELECT id FROM sso.roles WHERE name = 'admin')
  AND sso.role_permissions.permission_id IN (SELECT id FROM sso.permissions WHERE resource_type IN (2, 3, 4, 5, 6, 7));