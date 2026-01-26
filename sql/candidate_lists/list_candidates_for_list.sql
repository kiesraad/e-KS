SELECT
    clp.position,
    p.id as "id: PersonId",
    p.gender as "gender: Gender",
    p.last_name,
    p.last_name_prefix,
    p.first_name,
    p.initials,
    p.date_of_birth,
    p.bsn,
    p.place_of_residence,
    p.country_of_residence,
    p.locality,
    p.postal_code,
    p.house_number,
    p.house_number_addition,
    p.street_name,
    p.created_at,
    p.updated_at
FROM candidate_lists_persons clp
JOIN persons p ON p.id = clp.person_id
WHERE clp.candidate_list_id = $1
ORDER BY clp.position ASC
