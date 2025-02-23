function look_at_player(name)
	local player = get_player(name)
	if player then
		player.position.y = player.position.y + 1
		client:look_at(player.position)
	else
		client:chat(string.format("/w %s player not found!", sender))
	end
end

function go_to_player(name, opts)
	local player = get_player(name)
	if player then
		client:go_to(player.position, opts)
	else
		client:chat(string.format("/w %s player not found!", sender))
	end
end
