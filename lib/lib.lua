function clock_gettime(clock)
	local status, module = pcall(require, "posix")
	posix = status and module or nil

	if posix then
		local s, ns = posix.clock_gettime(clock)
		return s + ns / (10 ^ (math.floor(math.log10(ns)) + 1))
	else
		warn("failed to load posix module! falling back to os.time()")
		return os.time()
	end
end

function distance(p1, p2)
	return math.sqrt((p2.x - p1.x) ^ 2 + (p2.y - p1.y) ^ 2 + (p2.z - p1.z) ^ 2)
end

function table.shallow_copy(t)
	local t2 = {}
	for k, v in pairs(t) do
		t2[k] = v
	end
	return t2
end

function table.map(t, f)
	local t2 = {}
	for k, v in pairs(t) do
		t2[k] = f(v)
	end
	return t2
end

function table.contains(t, target)
	for _, v in pairs(t) do
		if v == target then
			return true
		end
	end
	return false
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

function dumpp(object, level)
	if not level then
		level = 0
	end
	if type(object) == "table" then
		local string = "{\n" .. string.rep("  ", level + 1)
		local parts = {}
		for key, value in pairs(object) do
			table.insert(parts, key .. " = " .. dumpp(value, level + 1))
		end
		string = string .. table.concat(parts, ",\n" .. string.rep("  ", level + 1))
		return string .. "\n" .. string.rep("  ", level) .. "}"
	else
		return tostring(object)
	end
end
