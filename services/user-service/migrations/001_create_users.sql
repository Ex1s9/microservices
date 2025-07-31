CREATE TYPE user_role AS ENUM ('player', 'developer', 'admin');

CREATE TABLE users (
     id UUID PRIMARY KEY,
     email VARCHAR(255) UNIQUE NOT NULL,
     username VARCHAR(100) UNIQUE NOT NULL,
     password_hash VARCHAR(255) NOT NULL,
     role user_role NOT NULL DEFAULT 'player',
     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
     updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
     wallet_balance DECIMAL(10, 2) NOT NULL DEFAULT 0.00
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);