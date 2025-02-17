function look_at_player(name)
	local player = get_player(name)
	if player then
		player.position.y = player.position.y + 1
		client:look_at(player.position)
	else
		client:chat("player not found!")
	end
end

function goto_player(name)
	local player = get_player(name)
	if player then
		client:goto(player.position)
	else
		client:chat("player not found!")
	end
end
