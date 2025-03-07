function look_at_player(name)
	local player = get_player(name)
	if player then
		player.position.y = player.position.y + 1
		client:look_at(player.position)
	end
end

function go_to_player(name, go_to_opts)
	client:go_to(get_player(name).position, go_to_opts)
end
