-- ===============================================
-- 0001_initial.up.sql
-- Initial application schema
-- ===============================================

CREATE EXTENSION IF NOT EXISTS btree_gist;

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

CREATE TABLE roles (
    role_id UUID PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(role_id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, role_id)
);

CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);

INSERT INTO roles (role_id, name)
VALUES
    ('30000000-0000-0000-0000-000000000001', 'admin'),
    ('30000000-0000-0000-0000-000000000002', 'user');

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
    article_id UUID NOT NULL REFERENCES article(article_id) ON DELETE CASCADE,
    entry_type VARCHAR(16) NOT NULL
        CHECK (entry_type IN ('availability', 'block')),
    block_reason VARCHAR(16)
        CHECK (block_reason IN ('vacation', 'repair')),
    summary VARCHAR(255) NOT NULL,
    location VARCHAR(255),
    description TEXT,
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE NOT NULL,
    rrule TEXT,
    dtstamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    source VARCHAR(16) NOT NULL DEFAULT 'manual'
        CHECK (source IN ('manual', 'import', 'system')),
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_by UUID,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT event_calendar_time_order_check
        CHECK (end_time > start_time),
    CONSTRAINT event_calendar_block_reason_semantics_check
        CHECK (
            (entry_type = 'availability' AND block_reason IS NULL)
            OR
            (entry_type = 'block' AND block_reason IN ('vacation', 'repair'))
        )
);

CREATE INDEX idx_event_calendar_article_start_time
    ON event_calendar(article_id, start_time);

CREATE INDEX idx_event_calendar_article_end_time
    ON event_calendar(article_id, end_time);

CREATE TABLE booking (
    booking_id UUID PRIMARY KEY,
    article_id UUID NOT NULL REFERENCES article(article_id) ON DELETE CASCADE,
    requested_by UUID REFERENCES users(id),
    requester_name VARCHAR(128),
    requester_email VARCHAR(128),
    note TEXT,
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'requested'
        CHECK (status IN ('requested', 'confirmed', 'rejected', 'cancelled', 'completed')),
    approved_by UUID REFERENCES users(id),
    approved_at TIMESTAMPTZ,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_by UUID,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT booking_time_order_check
        CHECK (end_time > start_time),
    CONSTRAINT booking_confirmed_no_overlap
        EXCLUDE USING GIST (
            article_id WITH =,
            tstzrange(start_time, end_time, '[)') WITH &&
        )
        WHERE (status = 'confirmed')
);

CREATE INDEX idx_booking_article_start_time
    ON booking(article_id, start_time);

CREATE INDEX idx_booking_article_status
    ON booking(article_id, status);

CREATE TABLE user_auth (
    user_id UUID PRIMARY KEY,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
