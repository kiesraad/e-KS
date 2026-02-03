SELECT
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
WHERE id NOT IN (
    SELECT person_id
    FROM candidate_lists_persons
    WHERE candidate_list_id = $1
)
ORDER BY last_name asc, initials asc
