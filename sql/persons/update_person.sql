UPDATE persons
SET
    gender = $1,
    last_name = $2,
    last_name_prefix = $3,
    first_name = $4,
    initials = $5,
    date_of_birth = $6,
    bsn = $7,
    place_of_residence = $8,
    country_of_residence = $9,
    updated_at = NOW()
WHERE id = $10
RETURNING
    id,
    gender as "gender: Gender",
    last_name,
    last_name_prefix,
    first_name,
    initials,
    date_of_birth,
    bsn,
    place_of_residence,
    country_of_residence,
    locality,
    postal_code,
    house_number,
    house_number_addition,
    street_name,
    created_at,
    updated_at
