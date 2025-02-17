local center = { x = 0, z = 0 }
local radius = 50

function on_tick()
	local entities = client:find_entities(function(e)
		return e.kind == "minecraft:player"
			and e.position.x > center.x - radius + 1
			and e.position.x < center.x + radius
			and e.position.z > center.z - radius
			and e.position.z < center.z + radius
	end)
	for _, e in entities do
		client:chat(string.format("%s (%s) at %.1f %.1f %.1f", e.kind, e.id, e.position.x, e.position.y, e.position.z))
	end
end

function on_init()
	info("client initialized, setting client information")
	client:set_client_information({ view_distance = 16 })
end
