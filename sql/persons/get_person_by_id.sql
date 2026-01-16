SELECT
    id,
    gender as "gender?: Gender",
    last_name,
    last_name_prefix,
    first_name,
    initials,
    date_of_birth,
    bsn,
    locality,
    postal_code,
    house_number,
    house_number_addition,
    street_name,
    is_dutch,
    custom_country,
    custom_region,
    address_line_1,
    address_line_2,
    created_at,
    updated_at
FROM persons
WHERE id = $1
