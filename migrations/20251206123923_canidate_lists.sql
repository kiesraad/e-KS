
CREATE TYPE electoral_district AS ENUM ('DR', 'FL', 'FR', 'GE', 'GR', 'LI', 'NB', 'NH', 'OV', 'UT', 'ZE', 'ZH', 'BO', 'SE', 'SA', 'KN');

CREATE TABLE candidate_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    electoral_districts electoral_district[] NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE candidate_lists_persons (
    candidate_list_id UUID REFERENCES candidate_lists(id) ON DELETE RESTRICT,
    person_id UUID REFERENCES persons(id) ON DELETE RESTRICT,
    position INTEGER NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
