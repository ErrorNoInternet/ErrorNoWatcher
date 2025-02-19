function get_player(name)
	local target_uuid = nil
	for uuid, player in client.tab_list do
		if player.name == name then
			target_uuid = uuid
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
