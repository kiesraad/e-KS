CREATE TABLE political_groups
(
    id                     UUID PRIMARY KEY,
    long_list_allowed      BOOLEAN,
    legal_name             VARCHAR(255),
    display_name           VARCHAR(255),
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
    locality              VARCHAR(255),
    postal_code           VARCHAR(16),
    house_number          VARCHAR(16),
    house_number_addition VARCHAR(16),
    street_name           VARCHAR(255),

    created_at            timestamp with time zone NOT NULL,
    updated_at            timestamp with time zone NOT NULL
);
