CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username      TEXT NOT NULL UNIQUE,
    email         TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE posts (
    id         UUID PRIMARY KEY,
    author_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title      TEXT NOT NULL,
    slug       TEXT NOT NULL UNIQUE,
    excerpt    TEXT NOT NULL DEFAULT '',
    body       TEXT NOT NULL,
    published  BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX posts_published_created_idx ON posts (published, created_at DESC);
CREATE INDEX posts_author_idx ON posts (author_id, updated_at DESC);
