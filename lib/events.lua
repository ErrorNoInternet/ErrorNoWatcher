Center = { x = 0, y = 64, z = 0 }
Radius = 100
Whitelist = table.shallow_copy(Owners)
Ticks = -1

function check_radius()
	Ticks = Ticks + 1
	if Ticks % 20 ~= 0 then
		return
	end

	local self_id = client.id
	local players = client:find_players(function(p)
		return self_id ~= p.id
			and p.position.x > Center.x - Radius + 1
			and p.position.x < Center.x + Radius
			and p.position.z > Center.z - Radius
			and p.position.z < Center.z + Radius
	end)

	local tab_list = client.tab_list
	for _, player in ipairs(players) do
		local target
		for _, tab_player in ipairs(tab_list) do
			if tab_player.uuid == player.uuid and not table.contains(Whitelist, tab_player.name) then
				target = tab_player
				break
			end
		end
		if not target then
			goto continue
		end

		client:chat(
			string.format(
				"%s is %s %d blocks away at %.2f %.2f %.2f facing %.2f %.2f",
				target.name,
				POSE_NAMES[player.pose + 1],
				distance(Center, player.position),
				player.position.x,
				player.position.y,
				player.position.z,
				player.direction.x,
				player.direction.y
			)
		)

		::continue::
	end
end

function update_listeners()
	for type, listeners in pairs(get_listeners()) do
		for id, _ in pairs(listeners) do
			remove_listeners(type, id)
		end
	end

	for type, listeners in pairs({
		login = {
			message = function()
				info("bot successfully logged in!")
			end,
			eat = function()
				sleep(5000)
				check_food()
			end,
		},
		death = {
			warn_bot_died = function()
				warn(
					string.format(
						"bot died at %.2f %.2f %.2f facing %.2f %.2f!",
						client.position.x,
						client.position.y,
						client.position.z,
						client.direction.x,
						client.direction.y
					)
				)
			end,
		},
		set_health = { auto_eat = check_food },
		tick = { log_player_positions = check_radius },
	}) do
		for id, callback in pairs(listeners) do
			add_listener(type, callback, id)
		end
	end
end
