insert or replace into water_systems (
    water_system_no, 
    name, 
    state_code, 
    is_no,
    created
)
values (
    :water_system_no, 
    :water_system_name, 
    :state_code, 
	:is_no,
    :created_timestamp
);
   
