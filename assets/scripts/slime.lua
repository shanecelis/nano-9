
anim_speed = 8
slime_animate = {
    idle = {
        index = 0,
        count = 6,
        loop = true
    },
    Hit={
        index=7,
        count=4,
        loop = false,
        next_anim = "idle"
    },
    death = {
        index = 0,
        count = 7,
        loop = false

    }
}

function slime_new()
    -- load slime
    slime_idle = image:load("images/slime/slime.png")
    slime_idle:set_grid(16,16,7,3)
    local s1 = slime_idle:spr(0)
    --s1.y = ground
    s1.anchor = { 0, -1 }
    return {
        sprite = s1,
        health = 2,
        cur_anim = "idle",
        anim_start = 0
    }
end

function slime_update(s)
    local count = slime_animate[s.cur_anim].count
    local loop = slime_animate[s.cur_anim].loop
    local start_index = slime_animate[s.cur_anim].index
    local t = time() - s.anim_start
    local frame = (t * anim_speed)
    if loop then
        s.sprite.index = start_index + frame % count
    else
        if frame >= count then
            -- we're over the last frame. stop?
            if s.cur_anim == "death" then
                table_remove_eq(slimes, s)
                s.sprite:despawn()
            else
                local next = slime_animate[s.cur_anim].next_anim
                if next then
                    s.cur_anim = next
                end
            end
        else
            s.sprite.index = start_index + frame % count
        end
    end
end

function slime_hit(s)
    s.health=s.health-1
    if s.health > 0 then
        s.cur_anim = "Hit"
        s.anim_start = time()
    elseif not s.dead then
        -- We dead.
        -- Play death animation.
        slime_death = image:load("images/slime/death.png")
        slime_death:set_grid(20,20,7,1)
        local death_sprite = slime_death:spr(0)
        death_sprite.anchor = { 0, -1 }
        death_sprite.x = s.sprite.x
        death_sprite.y = s.sprite.y
        s.sprite:despawn()
        s.sprite = death_sprite
        s.cur_anim = "death"
        s.anim_start = time()
        s.dead = true
    end
end
