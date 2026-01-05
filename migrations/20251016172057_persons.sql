
CREATE TYPE gender AS ENUM ('male', 'female', 'x');

CREATE TABLE persons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    last_name VARCHAR NOT NULL,
    initials VARCHAR NOT NULL,
    first_name VARCHAR,
    gender gender,
    date_of_birth DATE,
    locality VARCHAR,
    postal_code VARCHAR,
    house_number VARCHAR,
    house_number_addition VARCHAR,
    street_name VARCHAR,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
