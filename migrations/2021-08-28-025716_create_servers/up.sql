-- Your SQL goes here
CREATE TABLE servers (
  id BIGINT PRIMARY KEY,
  prefix TEXT NOT NULL,
  react BOOLEAN NOT NULL DEFAULT 'f'
)