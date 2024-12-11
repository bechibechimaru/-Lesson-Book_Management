INSERT INTO 
    roles (name)
VALUES  
    ('Admin'),
    ('User')
ON CONFLICT DO NOTHING;

INSERT INTO
    users (name, email, password_hash, role_id)
SELECT 
    'Shogo Karube',
    'mebiusu1968@gmail.com',
    '$2b$12$rRJhvErFgMUzMJG2wdRYq.7hunA2AnJVIwdCC7T7RZsTppgBvWL2C',
    role_id
FROM 
    roles
WHERE 
    name LIKE 'Admin';