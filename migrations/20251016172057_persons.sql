
CREATE TYPE gender AS ENUM ('male', 'female', 'x');

CREATE TABLE persons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- name
    last_name VARCHAR NOT NULL,
    last_name_prefix VARCHAR,
    initials VARCHAR NOT NULL,
    first_name VARCHAR,
    -- required personal details
    bsn CHAR(9),
    place_of_residence VARCHAR,
    country_of_residence VARCHAR,
    -- personal details
    gender gender,
    date_of_birth DATE,
    -- correspondence address
    locality VARCHAR,
    postal_code VARCHAR,
    house_number VARCHAR,
    house_number_addition VARCHAR,
    street_name VARCHAR,
    -- metadata
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

