function hold_items_in_hotbar(target_kinds, inventory)
	if not inventory then
		inventory = client:open_inventory()
	end
	for index, item in ipairs(inventory.contents) do
		if index >= 37 and index <= 45 and table.contains(target_kinds, item.kind) then
			inventory = nil
			sleep(500)
			client:set_held_slot(index - 37)
			return true
		end
	end
	return false
end

function hold_items(target_kinds)
	local inventory = client:open_inventory()
	if hold_items_in_hotbar(target_kinds, inventory) then
		return true
	end
	for index, item in ipairs(inventory.contents) do
		if table.contains(target_kinds, item.kind) then
			inventory:click({ source_slot = index - 1, target_slot = client.held_slot }, SWAP)
			sleep(100)
			inventory = nil
			sleep(500)
			return true
		end
	end
	inventory = nil
	sleep(500)
	return false
end

function steal(item_name)
	for _, chest_pos in ipairs(client:find_blocks(client.position, get_block_states({ "chest" }))) do
		client:go_to({ position = chest_pos, radius = 3 }, { type = RADIUS_GOAL })
		while client.pathfinder.is_calculating or client.pathfinder.is_executing do
			sleep(500)
		end
		client:look_at(chest_pos)

		local container = client:open_container_at(chest_pos)
		for index, item in ipairs(container.contents) do
			if item.kind == item_name then
				container:click({ slot = index - 1 }, THROW_ALL)
				sleep(50)
			end
		end

		container = nil
		while client.container do
			sleep(50)
		end
	end
end

function dump_inventory(hide_empty)
	local inventory = client:open_inventory()
	for index, item in ipairs(inventory.contents) do
		if hide_empty and item.count == 0 then
			goto continue
		end

		local item_damage = ""
		if item.damage then
			item_damage = item.damage
		end
		info(string.format("%-2d = %2dx %-32s %s", index - 1, item.count, item.kind, item_damage))

		::continue::
	end
end

function drop_all_hotbar()
	local inventory = client:open_inventory()
	for i = 0, 9 do
		inventory:click({ slot = 36 + i }, THROW_ALL)
	end
end

function drop_all_inventory()
	local inventory = client:open_inventory()
	for i = 0, 45 do
		inventory:click({ slot = i }, THROW_ALL)
	end
end
