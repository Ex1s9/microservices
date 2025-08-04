CREATE TYPE game_category AS ENUM (
     'unspecified',
     'action',
     'rpg',
     'strategy',
     'sports',
     'racing',
     'adventure',
     'simulation',
     'puzzle'
);

CREATE TYPE game_status AS ENUM (
     'draft',
     'under_review',
     'published',
     'suspended'
);

CREATE TABLE games (
     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
     name VARCHAR(255) NOT NULL,
     description TEXT NOT NULL,
     developer_id UUID NOT NULL,
     publisher_id UUID,
     cover_image VARCHAR(500) NOT NULL,
     trailer_url VARCHAR(500),
     release_date DATE NOT NULL,
     price DECIMAL(10, 2) NOT NULL CHECK (price >= 0),
     status game_status NOT NULL DEFAULT 'draft',
     
     categories game_category[] NOT NULL DEFAULT '{}',
     tags TEXT[] NOT NULL DEFAULT '{}',
     platforms TEXT[] NOT NULL DEFAULT '{}',
     screenshots TEXT[] NOT NULL DEFAULT '{}',
     
     rating_count INTEGER NOT NULL DEFAULT 0,
     average_rating DECIMAL(3, 2) NOT NULL DEFAULT 0.00 CHECK (average_rating >= 0 AND average_rating <= 5),
     purchase_count INTEGER NOT NULL DEFAULT 0,

     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
     updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
     deleted_at TIMESTAMPTZ,
     
     CONSTRAINT price_min CHECK (price >= 0),
     CONSTRAINT price_max CHECK (price <= 9999.99)
);

CREATE INDEX idx_games_developer_id ON games(developer_id);
CREATE INDEX idx_games_publisher_id ON games(publisher_id) WHERE publisher_id IS NOT NULL;
CREATE INDEX idx_games_status ON games(status);
CREATE INDEX idx_games_price ON games(price);
CREATE INDEX idx_games_created_at ON games(created_at);
CREATE INDEX idx_games_name_search ON games USING gin(to_tsvector('english', name));
CREATE INDEX idx_games_deleted_at ON games(deleted_at) WHERE deleted_at IS NULL;

-- Индексы для массивов
CREATE INDEX idx_games_categories ON games USING gin(categories);
CREATE INDEX idx_games_tags ON games USING gin(tags);
CREATE INDEX idx_games_platforms ON games USING gin(platforms);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
     NEW.updated_at = NOW();
     RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_games_updated_at BEFORE UPDATE
     ON games FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();