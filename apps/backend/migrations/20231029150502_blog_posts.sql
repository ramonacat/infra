CREATE TABLE posts (
    id UUID PRIMARY KEY,
    date_published TIMESTAMPTZ NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL
);