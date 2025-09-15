-- Add migration script here
DO $$ BEGIN
    CREATE TYPE role_type AS ENUM ('tutor', 'admin', 'helper');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;
CREATE TABLE IF NOT EXISTS roles
(
    id UUID NOT NULL PRIMARY KEY,
    username_id UUID NOT NULL references users(id),
    role role_type NOT NULL
);
CREATE INDEX roles_user_index ON roles(username_id);
