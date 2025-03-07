FishingBobber = nil
FishingTicks = 0
FishLastCaught = 0
LastEaten = 0

function auto_fish()
	stop_auto_fish()
	FishingTicks = 0

	function hold_fishing_rod()
		if client.held_item.kind == "minecraft:fishing_rod" or hold_items({ "minecraft:fishing_rod" }) then
			return true
		end
		warn("no fishing rod found!")
	end

	if not hold_fishing_rod() then
		return
	end

	add_listener("add_entity", function(entity)
		if entity.kind == "minecraft:fishing_bobber" and entity.data == client.id then
			FishingBobber = entity
		end
	end, "auto-fish_watch-bobber")

	add_listener("remove_entities", function(entity_ids)
		if table.contains(entity_ids, FishingBobber.id) then
			if os.time() - LastEaten < 3 then
				sleep(3000)
			end
			hold_fishing_rod()
			client:use_item()
		end
	end, "auto-fish_watch-bobber")

	add_listener("level_particles", function(particle)
		if particle.kind == 30 and particle.count == 6 then
			local current_bobber = client:find_entities(function(e)
				return e.id == FishingBobber.id
			end)[1]
			if distance(current_bobber.position, particle.position) <= 0.75 then
				FishLastCaught = os.time()
				client:use_item()
			end
		end
	end, "auto-fish")

	add_listener("tick", function()
		FishingTicks = FishingTicks + 1
		if FishingTicks % 600 ~= 0 then
			return
		end

		if os.time() - FishLastCaught >= 60 then
			hold_fishing_rod()
			client:use_item()
		end
	end, "auto-fish_watchdog")

	client:use_item()
end

function stop_auto_fish()
	remove_listeners("add_entity", "auto-fish_watch-bobber")
	remove_listeners("remove_entities", "auto-fish_watch-bobber")
	remove_listeners("level_particles", "auto-fish")
	remove_listeners("tick", "auto-fish_watchdog")

	if FishingBobber and client:find_entities(function(e)
		return e.id == FishingBobber.id
	end)[1] then
		FishingBobber = nil
		client:use_item()
	end
end

function attack_entities(target_kind, minimum)
	if not minimum then
		minimum = 0
	end

	function hold_sword()
		if client.held_item.kind == "minecraft:iron_sword" or hold_items({ "minecraft:iron_sword" }) then
			return true
		end
		warn("no sword found!")
	end

	while true do
		local self_pos = client.position
		local entities = client:find_entities(function(e)
			return e.kind == target_kind and distance(e.position, self_pos) < 5
		end)

		if #entities > minimum then
			local e = entities[1]
			local pos = e.position
			pos.y = pos.y + 1.5

			hold_sword()
			client:look_at(pos)
			client:attack(e.id)
			while client.has_attack_cooldown do
				sleep(100)
			end
		else
			sleep(1000)
		end
	end
end

function check_food(hunger)
	if hunger.food >= 20 then
		return
	end

	local current_time = os.time()
	if current_time - LastEaten >= 3 then
		LastEaten = current_time

		while not hold_items({
			"minecraft:golden_carrot",
			"minecraft:cooked_beef",
			"minecraft:bread",
		}) do
			sleep(1000)
			LastEaten = current_time
		end
		client:use_item()
	end
end
