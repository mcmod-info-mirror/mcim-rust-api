-- Project 表
CREATE TABLE modrinth_projects (
    id VARCHAR PRIMARY KEY,
    slug VARCHAR NOT NULL,
    title VARCHAR,
    description TEXT,
    categories TEXT[],
    client_side VARCHAR,
    server_side VARCHAR,
    body TEXT,
    status VARCHAR,
    requested_status VARCHAR,
    additional_categories TEXT[],
    issues_url VARCHAR,
    source_url VARCHAR,
    wiki_url VARCHAR,
    discord_url VARCHAR,
    donation_urls JSONB,
    project_type VARCHAR,
    downloads BIGINT,
    icon_url VARCHAR,
    color INTEGER,
    thread_id VARCHAR,
    monetization_status VARCHAR,
    team VARCHAR,
    body_url VARCHAR,
    published TIMESTAMPTZ,
    updated TIMESTAMPTZ,
    approved TIMESTAMPTZ,
    queued TIMESTAMPTZ,
    followers INTEGER,
    license JSONB,
    versions TEXT[],
    game_versions TEXT[],
    loaders TEXT[],
    gallery JSONB,
    sync_at TIMESTAMPTZ NOT NULL
);

-- Version 表
CREATE TABLE modrinth_versions (
    id VARCHAR PRIMARY KEY,
    project_id VARCHAR NOT NULL REFERENCES modrinth_projects(id),
    name VARCHAR,
    version_number VARCHAR,
    changelog TEXT,
    dependencies JSONB,
    game_versions TEXT[],
    version_type VARCHAR,
    loaders TEXT[],
    featured BOOLEAN,
    status VARCHAR,
    requested_status VARCHAR,
    author_id VARCHAR,
    date_published TIMESTAMPTZ,
    downloads BIGINT,
    changelog_url VARCHAR,
    files JSONB,
    sync_at TIMESTAMPTZ NOT NULL
);

-- File 表
CREATE TABLE modrinth_files (
    sha512 VARCHAR NOT NULL,
    sha1 VARCHAR NOT NULL,
    url VARCHAR NOT NULL,
    filename VARCHAR NOT NULL,
    is_primary BOOLEAN NOT NULL,
    size BIGINT NOT NULL,
    file_type VARCHAR,
    version_id VARCHAR NOT NULL REFERENCES modrinth_versions(id),
    project_id VARCHAR NOT NULL REFERENCES modrinth_projects(id),
    file_cdn_cached BOOLEAN,
    sync_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (sha512, sha1)
);

-- Category 表
CREATE TABLE modrinth_categories (
    icon TEXT,
    name VARCHAR PRIMARY KEY,
    project_type VARCHAR,
    header VARCHAR,
    sync_at TIMESTAMPTZ NOT NULL
);

-- Loader 表
CREATE TABLE modrinth_loaders (
    icon TEXT,
    name VARCHAR PRIMARY KEY,
    supported_project_types TEXT[],
    sync_at TIMESTAMPTZ NOT NULL
);

-- GameVersion 表
CREATE TABLE modrinth_game_versions (
    version VARCHAR PRIMARY KEY,
    version_type VARCHAR,
    date TIMESTAMPTZ,
    major BOOLEAN,
    sync_at TIMESTAMPTZ NOT NULL
);