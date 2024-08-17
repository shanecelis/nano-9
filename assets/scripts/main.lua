-- main.lua
--
-- An ok cat game
require("assets/scripts/utils")
-- walk speed
speed = 30
-- anim_speed is our animation speed. How many frames per second?
anim_speed = 8
animate = {
    right = {
        index = 8,
        count = 4
    },
    scratch = {
        index = 16,
        count = 3
    },
    jump = {
        index = 24,
        count = 1,
    }
}
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
ground = -64



function slime_new()
    -- load slime
    slime_idle = loadimg("images/slime/slime.png")
    slime_idle:set_grid(16,16,7,3)
    local s1 = slime_idle:spr(0)
    s1.y = ground
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
    else
        -- We dead.
        --Play death animation.
        slime_death = loadimg("images/slime/death.png")
        slime_death:set_grid(20,20,7,1)
        local death_sprite = slime_death:spr(0)
        death_sprite.anchor = { 0, -1 }
        death_sprite.x = s.sprite.x
        death_sprite.y = s.sprite.y
        s.sprite:despawn()
        s.sprite = death_sprite
        s.cur_anim = "death"
        s.anim_start = time()
    end
end

-- _init() is called once per load.
function _init()

    meow_sound = audio:load("audio/Cat Meow Short 09 .ogg")
    -- load the cat sprites
    cat_sprites = loadimg("images/Cat_Sprite.png")
    -- set up the sprite sheet
    cat_sprites:set_grid(32, 32, 4, 8)
    -- create the cat sprite
    s = cat_sprites:spr(8)
    -- s.y = ground        -- place cat on ground
    s.anchor = { 0, -0.8 } -- move the cat sprite by its bottom-center.

    -- load the hearts image
    hearts = loadimg("images/heart-of-a-thousand-miles.png");
    -- set up the sprite sheet
    hearts:set_grid(12, 12, 10, 10)
    -- create the first heart sprite
    h1 = hearts:spr(0)
    h1.anchor = { -1, 1 } -- move the heart sprite by its top-left.
    -- place heart on top left of screen.
    h1.x = -64
    h1.y = 64
    h2 = hearts:spr(1)
    h2.index = 0
    h2.anchor = { -1, 1 }
    h2.y = 64
    h2.x = -50

    s1 = slime_new()
    s2 = slime_new()
    s2.sprite.x = 22

    slimes = { s1, s2 }


    -- clear the screen
    cls(1)
end

-- _draw() is called once per frame, or 60 times a second.
function _draw()
    -- t is time in seconds.
    t = time()

    -- dt is the time since we last were here. Usually 1/60th of a second.
    dt = delta_time()

    -- Check if 'z' is pressed.
    if btnp(4) then
        scratch_start = t -- note cat scratch start time.
        if not ((last_meow or {}).is_playing) then
            last_meow = meow_sound:sfx()
        end
        for i,slime in ipairs(slimes) do
            distance = math.abs( s.x -slime.sprite.x )
            print("hit distance ", distance)
            if distance <= 17.5 then
                slime_hit(slime)
            end
        end
    end

    -- Check if 'x' is pressed.
    if btnp(5) then
        jump_start = t -- note cat scratch start time.
        s.y = s.y + 20
    end

    if scratch_start then
        -- cat is scratching

        -- scratch_frame is either 0, 1, 2 for the three frames.
        scratch_frame = (t - scratch_start) * anim_speed
        if scratch_frame >= animate.scratch.count then
            scratch_start = nil -- stop scratching
            s.index = 8         -- set our animation to 8
        else
            s.index = animate.scratch.index + scratch_frame
        end
    else
        if jump_start then

        end
        -- cat is not scratching
        if btn(0) then
            -- go left
            s.index = animate.right.index + t * anim_speed % animate.right.count
            s.x = s.x - speed * dt
            s.flip_x = true
        elseif btn(1) then
            -- go right
            s.index = animate.right.index + t * anim_speed % animate.right.count
            s.x = s.x + speed * dt
            s.flip_x = false
        end
    end

    --Gravity
    --if true stop at 64
    is_in_air = s.y > ground
    if is_in_air then
        s.y = s.y - speed * dt
    end

    -- animate slime
    for i,s in ipairs(slimes) do
        slime_update(s)
    end
end
