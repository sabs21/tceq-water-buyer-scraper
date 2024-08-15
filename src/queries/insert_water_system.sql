case when 
	not exists(select * from water_systems where water_system_no = :water_system_no)
then
	insert into water_systems (
		water_system_no, 
		name, 
		state_code, 
		is_no
	) values (
		:water_system_no, 
		:name, 
		:state_code, 
		:is_no
	)
end;