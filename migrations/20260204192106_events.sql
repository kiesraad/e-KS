CREATE TABLE streams (
  stream_id UUID PRIMARY KEY,
  last_event_id BIGINT NOT NULL
);

CREATE TABLE events (
  stream_id UUID NOT NULL,
  event_id BIGINT NOT NULL,
  created_at timestamp with time zone NOT NULL,
  payload jsonb NOT NULL,
  PRIMARY KEY (stream_id, event_id)
);
