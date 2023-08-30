CREATE TYPE language_type AS ENUM ('en', 'ru', 'rs');

CREATE TABLE preference (
    id         SERIAL PRIMARY KEY,
    chat_id    BIGINT NOT NULL,
    language   language_type NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TYPE message_type AS ENUM ('text', 'command');

CREATE TABLE messages (
    id         SERIAL PRIMARY KEY,
    chat_id    BIGINT NOT NULL,
    text       TEXT NOT NULL,
    type       message_type NOT NULL DEFAULT 'text',
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE subscriptions (
    id         SERIAL PRIMARY KEY,
    chat_id    BIGINT NOT NULL,
    address    TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
