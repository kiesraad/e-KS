SELECT
    id,
    gender as "gender: Gender",
    last_name,
    last_name_prefix,
    first_name,
    initials,
    date_of_birth,
    bsn,
    no_bsn_confirmed,
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
FROM persons
WHERE id = $1
