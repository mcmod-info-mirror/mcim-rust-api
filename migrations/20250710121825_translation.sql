-- Add migration script here
CREATE TABLE modrinth_translation (
    project_id VARCHAR PRIMARY KEY,
    translated TEXT NOT NULL,
    original TEXT NOT NULL,
    translated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE curseforge_translation (
    mod_id INTEGER PRIMARY KEY,
    translated TEXT NOT NULL,
    original TEXT NOT NULL,
    translated_at TIMESTAMPTZ NOT NULL
);