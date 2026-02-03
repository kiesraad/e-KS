CREATE TYPE gender AS ENUM ('male', 'female');

CREATE TABLE persons
(
    id                    UUID PRIMARY KEY,
    -- name
    last_name             VARCHAR(255)             NOT NULL,
    last_name_prefix      VARCHAR(50),
    initials              VARCHAR(50)              NOT NULL,
    first_name            VARCHAR(255),
    -- required personal details
    bsn                   VARCHAR(9),
    place_of_residence    VARCHAR(255),
    country_of_residence  VARCHAR(2),
    -- personal details
    gender                gender,
    date_of_birth         DATE,
    -- deputy agent
    representative_last_name             VARCHAR(255),
    representative_last_name_prefix      VARCHAR(50),
    representative_initials              VARCHAR(50),
    -- correspondence address
    locality              VARCHAR(255),
    postal_code           VARCHAR(16),
    house_number          VARCHAR(16),
    house_number_addition VARCHAR(16),
    street_name           VARCHAR(255),
    -- metadata
    created_at            timestamp with time zone NOT NULL,
    updated_at            timestamp with time zone NOT NULL
);

