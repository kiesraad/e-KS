UPDATE persons
SET
    representative_last_name = $1,
    representative_last_name_prefix = $2,
    representative_initials = $3,
    locality = $4,
    postal_code = $5,
    house_number = $6,
    house_number_addition = $7,
    street_name = $8,
    updated_at = NOW()
WHERE id = $9
RETURNING
    id,
    gender as "gender?: Gender",
    last_name,
    last_name_prefix,
    first_name,
    initials,
    date_of_birth,
    bsn,
    place_of_residence,
    country_of_residence,
    representative_last_name,
    representative_last_name_prefix,
    representative_initials,
    locality,
    postal_code,
    house_number,
    house_number_addition,
    street_name,
    created_at,
    updated_at
