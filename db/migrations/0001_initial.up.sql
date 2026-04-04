-- ===============================================
-- 0001_initial.up.sql
-- Initial application schema
-- ===============================================

CREATE TABLE users (
    id           UUID PRIMARY KEY,
    username     VARCHAR(64) NOT NULL UNIQUE,
    email        VARCHAR(128) NOT NULL,
    created_by   UUID,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_by  UUID,
    modified_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_email ON users(email);

CREATE TABLE towns (
    town_id UUID PRIMARY KEY,
    name VARCHAR(36) NOT NULL
);

CREATE TABLE categories (
    category_id UUID PRIMARY KEY,
    name VARCHAR(36) NOT NULL
);

CREATE TABLE article (
    article_id UUID PRIMARY KEY,
    name VARCHAR(36) NOT NULL,
    category UUID NOT NULL REFERENCES categories(category_id),
    description TEXT NOT NULL,
    town UUID NOT NULL REFERENCES towns(town_id),
    status VARCHAR(16) NOT NULL DEFAULT 'aktiv'
        CHECK (status IN ('aktiv', 'ausgedient')),
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_by UUID,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE uploaded_files (
    id UUID PRIMARY KEY,
    file_name VARCHAR(128) NOT NULL,
    origin_file_name VARCHAR(128) NOT NULL,
    file_relative_path VARCHAR(256) NOT NULL,
    file_url VARCHAR(256) NOT NULL,
    content_type VARCHAR(64) NOT NULL,
    file_size BIGINT NOT NULL,
    file_type VARCHAR(16) NOT NULL,
    article_id UUID REFERENCES article(article_id) ON DELETE SET NULL,
    sort_order INT NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    is_cover BOOLEAN NOT NULL DEFAULT FALSE,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_by UUID,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_uploaded_files_article_id ON uploaded_files(article_id);

CREATE TABLE event_calendar (
    event_id UUID PRIMARY KEY,
    article_id UUID REFERENCES article(article_id),
    booking_type VARCHAR(16) NOT NULL
        CHECK (booking_type IN ('buchbar', 'urlaub', 'reparatur')),
    summary VARCHAR(255) NOT NULL,
    location VARCHAR(255),
    description TEXT,
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE NOT NULL,
    rrule TEXT,
    dtstamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_by UUID,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE user_auth (
    user_id UUID PRIMARY KEY,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
