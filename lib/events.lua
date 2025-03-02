local center = { x = 0, z = 0 }
local radius = 100

function log_player_positions()
	local entities = client:find_entities(function(e)
		return e.kind == "minecraft:player"
			and e.position.x > center.x - radius + 1
			and e.position.x < center.x + radius
			and e.position.z > center.z - radius
			and e.position.z < center.z + radius
	end)
	for _, e in ipairs(entities) do
		client:chat(string.format("%s (%s) at %.1f %.1f %.1f", e.kind, e.id, e.position.x, e.position.y, e.position.z))
	end
end

add_listener("init", function()
	info("client initialized, setting information...")
	client:set_client_information({ view_distance = 16 })
end)

add_listener("login", function()
	info("player successfully logged in!")
end)

add_listener("death", function()
	warn(string.format("player died at %.1f %.1f %.1f!", client.position.x, client.position.y, client.position.z))
end, "warn_player_died")

add_listener("tick", log_player_positions)
