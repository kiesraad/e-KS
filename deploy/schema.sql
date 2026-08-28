-- Full application database schema. Mirrors the runtime migration in
-- src/store/database.rs plus the ACME challenge storage.

-- Event stream bookkeeping: one row per (stream, election) pair.
CREATE TABLE IF NOT EXISTS streams (
  stream_id UUID NOT NULL,
  election TEXT NOT NULL,
  last_event_id BIGINT NOT NULL,
  scope TEXT NOT NULL DEFAULT 'political_group',
  encrypted_key BYTEA,
  PRIMARY KEY (stream_id, election)
);

-- Encrypted, hash-chained event log.
CREATE TABLE IF NOT EXISTS events (
  stream_id UUID NOT NULL,
  election TEXT NOT NULL,
  event_id BIGINT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL,
  hash BYTEA NOT NULL,
  payload BYTEA NOT NULL,
  PRIMARY KEY (stream_id, election, event_id)
);
-- Supports looking up an event (and its stream) by its chain hash.
CREATE INDEX IF NOT EXISTS events_hash_idx ON events(hash);

-- User sessions. `token` holds the token's SHA-256 hash, not the token itself.
CREATE TABLE IF NOT EXISTS sessions (
  token TEXT PRIMARY KEY,
  stream_id UUID,
  paper_correction_stream_id UUID,
  current_election JSONB,
  locale TEXT NOT NULL,
  last_activity TIMESTAMPTZ NOT NULL,
  saml_name_id TEXT NOT NULL DEFAULT '',
  scope TEXT NOT NULL DEFAULT 'political_group',
  csb_user JSONB,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  user_agent_hash TEXT,
  csrf_token TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS sessions_last_activity_idx
  ON sessions(last_activity);

-- In-flight request deduplication.
CREATE TABLE IF NOT EXISTS pending_requests (
  id TEXT PRIMARY KEY,
  created_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX IF NOT EXISTS pending_requests_created_at_idx
  ON pending_requests(created_at);

-- http-01 challenge tokens, shared so any instance can answer a validation
-- request. The key authorization is public by protocol.
CREATE TABLE IF NOT EXISTS acme_challenges (
  token TEXT PRIMARY KEY,
  key_authorization TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS acme_challenges_created_at_idx
  ON acme_challenges(created_at);
