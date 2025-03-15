SpeedTracking = {}
TpsTracking = {}

function entity_speed(uuid, seconds)
	if not seconds then
		seconds = 1
	end

	local callback = function()
		local old_entity = SpeedTracking[uuid]
		local new_entity = client:find_entities(function(e)
			return e.uuid == uuid
		end)[1]

		if not new_entity then
			remove_listeners("tick", "speed-tracking_" .. uuid)
			SpeedTracking[uuid] = -1
			return
		end

		if old_entity then
			old_entity._distance = old_entity._distance + distance(old_entity.position, new_entity.position)
			old_entity.position = new_entity.position

			if old_entity._ticks < seconds * 20 then
				old_entity._ticks = old_entity._ticks + 1
			else
				remove_listeners("tick", "speed-tracking_" .. uuid)
				SpeedTracking[uuid] = old_entity._distance / seconds
			end
		else
			new_entity._ticks = 1
			new_entity._distance = 0
			SpeedTracking[uuid] = new_entity
		end
	end
	add_listener("tick", callback, "speed-tracking_" .. uuid)

	repeat
		sleep(seconds * 1000 / 10)
	until type(SpeedTracking[uuid]) == "number"

	local speed = SpeedTracking[uuid]
	SpeedTracking[uuid] = nil
	return speed
end

function tps(ms)
	if not ms then
		ms = 1000
	end

	add_listener("tick", function()
		if not TpsTracking.ticks then
			TpsTracking.ticks = 0
			sleep(ms)
			TpsTracking.result = TpsTracking.ticks
			remove_listeners("tick", "tps_tracking")
		else
			TpsTracking.ticks = TpsTracking.ticks + 1
		end
	end, "tps_tracking")

	sleep(ms)
	repeat
		sleep(20)
	until TpsTracking.result

	local tps = TpsTracking.result / (ms / 1000)
	TpsTracking = {}
	return tps
end

function nether_travel(pos, go_to_opts)
	info(string.format("going to %.2f %.2f %.2f through nether", pos.x, pos.y, pos.z))

	local portal_block_states = get_block_states({ "nether_portal" })
	local nether_pos = table.shallow_copy(pos)
	nether_pos.x = nether_pos.x / 8
	nether_pos.z = nether_pos.z / 8

	if client.dimension == "minecraft:overworld" then
		info("currently in overworld, finding nearest portal")
		local portals = client:find_blocks(client.position, portal_block_states)

		info(string.format("going to %.2f %.2f %.2f through nether", portals[1].x, portals[1].y, portals[1].z))
		client:go_to(portals[1], go_to_opts)
		while client.dimension ~= "minecraft:the_nether" do
			sleep(1000)
		end
		sleep(3000)
	end

	info(string.format("currently in nether, going to %.2f %.2f", nether_pos.x, nether_pos.z))
	client:go_to(nether_pos, { type = XZ_GOAL })
	while client.pathfinder.is_calculating or client.pathfinder.is_executing do
		sleep(1000)
	end

	info("arrived, looking for nearest portal")
	local portals_nether = client:find_blocks(client.position, portal_block_states)
	if not next(portals_nether) then
		warn("failed to find portals in the nether")
		return
	end

	local found_portal = false
	for _, portal in ipairs(portals_nether) do
		if (client.position.y > 127) == (portal.y > 127) then
			found_portal = true

			info(string.format("found valid portal, going to %.2f %.2f %.2f", portal.x, portal.y, portal.z))
			client:go_to(portal)
			while client.dimension ~= "minecraft:overworld" do
				sleep(1000)
			end
			sleep(3000)
		end

		if found_portal then
			break
		end
	end
	if not found_portal then
		warn("failed to find valid portals in the nether")
		return
	end

	info(string.format("back in overworld, going to %.2f %.2f %.2f", pos.x, pos.y, pos.z))
	client:go_to(pos, go_to_opts)
end

function interact_bed()
	local bed = client:find_blocks(
		client.position,
		get_block_states({
			"brown_bed",
			"white_bed",
			"yellow_bed",
		})
	)[1]
	if not bed then
		return
	end

	client:go_to({ position = bed, radius = 2 }, { type = RADIUS_GOAL, options = { without_mining = true } })
	while client.pathfinder.is_calculating or client.pathfinder.is_executing do
		sleep(500)
	end

	client:look_at(bed)
	client:block_interact(bed)
end

function closest_entity(target_kind)
	local self_pos = client.position
	local entities = client:find_entities(function(e)
		return e.kind == target_kind
	end)

	local closest_entity = entities[1]
	local closest_distance = distance(closest_entity.position, self_pos)
	for _, entity in ipairs(entities) do
		local dist = distance(entity.position, self_pos)
		if dist <= closest_distance then
			closest_entity = entity
			closest_distance = dist
		end
	end
	return closest_entity
end

function get_player(name)
	local target_uuid = nil
	for _, player in ipairs(client.tab_list) do
		if player.name == name then
			target_uuid = player.uuid
			break
		end
	end

	return client:find_entities(function(e)
		return e.kind == "minecraft:player" and e.uuid == target_uuid
	end)[1]
end

function distance(p1, p2)
	return math.sqrt((p2.x - p1.x) ^ 2 + (p2.y - p1.y) ^ 2 + (p2.z - p1.z) ^ 2)
end

function dump(object)
	if type(object) == "table" then
		local string = "{ "
		local parts = {}
		for key, value in pairs(object) do
			table.insert(parts, key .. " = " .. dump(value))
		end
		string = string .. table.concat(parts, ", ")
		return string .. " " .. "}"
	else
		return tostring(object)
	end
end

function dump_pretty(object, level)
	if not level then
		level = 0
	end
	if type(object) == "table" then
		local string = "{\n" .. string.rep("  ", level + 1)
		local parts = {}
		for key, value in pairs(object) do
			table.insert(parts, key .. " = " .. dump_pretty(value, level + 1))
		end
		string = string .. table.concat(parts, ",\n" .. string.rep("  ", level + 1))
		return string .. "\n" .. string.rep("  ", level) .. "}"
	else
		return tostring(object)
	end
end
