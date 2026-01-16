UPDATE persons
SET
    gender = $1,
    last_name = $2,
    last_name_prefix = $3,
    first_name = $4,
    initials = $5,
    date_of_birth = $6,
    bsn = $7,
    updated_at = NOW()
WHERE id = $8
RETURNING
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
