CREATE TABLE political_groups
(
    id           UUID PRIMARY KEY         DEFAULT gen_random_uuid(),
    legal_name   VARCHAR                                            NOT NULL,
    display_name VARCHAR                                            NOT NULL,
    created_at   timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at   timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE authorised_agents
(
    id                    UUID PRIMARY KEY         DEFAULT gen_random_uuid(),
    political_group_id    UUID REFERENCES political_groups (id) ON DELETE CASCADE,

    last_name             VARCHAR                                            NOT NULL,
    last_name_prefix      VARCHAR,
    initials              VARCHAR                                            NOT NULL,

    locality              VARCHAR,
    postal_code           VARCHAR,
    house_number          VARCHAR,
    house_number_addition VARCHAR,
    street_name           VARCHAR,

    created_at            timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at            timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


CREATE TABLE list_submitters
(
    id                    UUID PRIMARY KEY         DEFAULT gen_random_uuid(),
    political_group_id    UUID REFERENCES political_groups (id) ON DELETE CASCADE,

    last_name             VARCHAR                                            NOT NULL,
    last_name_prefix      VARCHAR,
    initials              VARCHAR                                            NOT NULL,

    -- postal address (must be Dutch)
    locality              VARCHAR,
    postal_code           VARCHAR,
    house_number          VARCHAR,
    house_number_addition VARCHAR,
    street_name           VARCHAR,

    created_at            timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at            timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

ALTER TABLE political_groups
    -- Indicates the current list submitter
    -- Substitute submitters are the remaining people in the 'list_submitters' table
    ADD COLUMN list_submitter_id   UUID REFERENCES list_submitters (id),
    ADD COLUMN authorised_agent_id UUID REFERENCES authorised_agents (id);
