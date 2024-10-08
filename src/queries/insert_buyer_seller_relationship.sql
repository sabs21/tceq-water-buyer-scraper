insert or replace into water_buyer_relationships (
    seller,
    buyer,
    population,
    availability,
    created
)
values (
    :seller,
    :buyer,
    :population,
    (select id from availability_codes where code = :availability),
    :created_timestamp
);
