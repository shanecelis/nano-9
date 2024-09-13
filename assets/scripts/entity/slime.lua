require("assets/scripts/slime")
print("slime entity loaded")
function _init()
    print("slime entity inited")
    s = slime_new()

    local Transform = world:get_type_by_name("Transform")
    local t = world:get_component(entity,Transform)
    s.sprite.x = t.translation.x
    s.sprite.y = t.translation.y
    s.sprite.z = t.translation.z
    print("set slime position.")
end

function _update()
    slime_update(s)
end
