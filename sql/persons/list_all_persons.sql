SELECT
    id,
    gender as "gender: Gender",
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
    address_line_1,
    address_line_2,
    is_dutch,
    custom_country,
    custom_region,
    created_at,
    updated_at
FROM persons
ORDER BY last_name asc, initials asc
