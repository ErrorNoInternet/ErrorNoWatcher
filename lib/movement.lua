function look_at_player(name)
	local player = get_player(name)
	if player then
		player.position.y = player.position.y + 1
		client:look_at(player.position)
	else
		client:chat(string.format("/w %s player not found!", sender))
	end
end

function goto_player(name, opts)
	local player = get_player(name)
	if player then
		client:goto(player.position, opts)
	else
		client:chat(string.format("/w %s player not found!", sender))
	end
end
