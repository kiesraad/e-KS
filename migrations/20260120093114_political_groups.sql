CREATE TABLE political_groups
(
    id                     UUID PRIMARY KEY,
    long_list_allowed      BOOLEAN,
    legal_name             VARCHAR(255)             NOT NULL,
    legal_name_confirmed   BOOLEAN,
    display_name           VARCHAR(255)             NOT NULL,
    display_name_confirmed BOOLEAN,
    created_at             timestamp with time zone NOT NULL,
    updated_at             timestamp with time zone NOT NULL
);

CREATE TABLE authorised_agents
(
    id                    UUID PRIMARY KEY,
    political_group_id    UUID                     NOT NULL REFERENCES political_groups (id) ON DELETE CASCADE,

    last_name             VARCHAR(255)             NOT NULL,
    last_name_prefix      VARCHAR(50),
    initials              VARCHAR(50)              NOT NULL,

    locality              VARCHAR(255)             NOT NULL,
    postal_code           VARCHAR(16)              NOT NULL,
    house_number          VARCHAR(16)              NOT NULL,
    house_number_addition VARCHAR(16),
    street_name           VARCHAR(255)             NOT NULL,

    created_at            timestamp with time zone NOT NULL,
    updated_at            timestamp with time zone NOT NULL
);


CREATE TABLE list_submitters
(
    id                    UUID PRIMARY KEY,
    political_group_id    UUID                     NOT NULL REFERENCES political_groups (id) ON DELETE CASCADE,

    last_name             VARCHAR(255)             NOT NULL,
    last_name_prefix      VARCHAR(50),
    initials              VARCHAR(50)              NOT NULL,

    -- postal address (must be Dutch)
    locality              VARCHAR(255)             NOT NULL,
    postal_code           VARCHAR(16)              NOT NULL,
    house_number          VARCHAR(16)              NOT NULL,
    house_number_addition VARCHAR(16),
    street_name           VARCHAR(255)             NOT NULL,

    created_at            timestamp with time zone NOT NULL,
    updated_at            timestamp with time zone NOT NULL
);

ALTER TABLE political_groups
    -- Indicates the current list submitter
    -- Substitute submitters are the remaining people in the 'list_submitters' table
    ADD COLUMN list_submitter_id   UUID REFERENCES list_submitters (id),
    ADD COLUMN authorised_agent_id UUID REFERENCES authorised_agents (id);
