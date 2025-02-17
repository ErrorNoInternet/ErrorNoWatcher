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
