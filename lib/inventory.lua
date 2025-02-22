function steal(item_name)
    for _, chest_pos in client:find_blocks(client.position, get_block_states({ "chest" })) do
        client:chat(dump(chest_pos))

        client:goto({ position = chest_pos, radius = 3 }, { type = RADIUS_GOAL })
        while client.pathfinder.is_calculating or client.pathfinder.is_executing do
            sleep(50)
        end
        client:look_at(chest_pos)

        local container = client:open_container_at(chest_pos)
        for index, item in container.contents do
            if item.kind == item_name then
                container:click({slot = index - 1}, THROW_ALL)
                sleep(50)
            end
        end

        container = nil
        while client.open_container do
            sleep(50)
        end
    end
end

function drop_all_hotbar()
    local inventory = client:open_inventory()
    for i = 0, 9 do
        inventory:click({slot = 36 + i}, THROW_ALL)
    end
end

function drop_all_inventory()
    local inventory = client:open_inventory()
    for i = 0, 45 do
        inventory:click({slot = i}, THROW_ALL)
    end
end
