-- main.lua
--
-- An ok cat game
require("assets/scripts/utils")
require("assets/scripts/slime")
features = {
    up_down = true,
    jump = false,
}
-- walk speed
speed = 30
-- anim_speed is our animation speed. How many frames per second?
anim_speed = 8
ground = -64
animate = {
    right = {
        index = 8,
        count = 4
    },
    down = {
        index = 4,
        count = 4
    },
    up = {
        index = 12,
        count = 3
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

-- _init() is called once per load.
function _init()
    --text.default:print("Hello world")
    music = audio:load("music/Comfortable Mystery.ogg")
    m = music:play_loop()
    meow_sound = audio:load("audio/Cat Meow Short 09.ogg")
    -- load the cat sprites
    cat_sprites = image:load("images/Cat_Sprite.png")
    -- set up the sprite sheet
    cat_sprites:set_grid(32, 32, 4, 8)
    -- create the cat sprite
    s = cat_sprites:spr(8)
    -- s.y = ground        -- place cat on ground
    s.anchor = { 0, -0.8 } -- move the cat sprite by its bottom-center.

    -- load the hearts image
    hearts = image:load("images/heart-of-a-thousand-miles.png");
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
            paw_x = s.x + 16
            distance = math.abs( paw_x - slime.sprite.x )
            print("hit distance ", distance)
            if distance <= 1.5 then
                slime_hit(slime)
            end
        end
    end

    -- Check if 'x' is pressed.
    --if btnp(5) then
        --jump_start = t -- note cat scratch start time.
        --s.y = s.y + 20
        -- end

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
        elseif btn(2) then
            -- go up
            s.index = animate.up.index + t * anim_speed % animate.up.count
            s.y = s.y + speed * dt
        elseif btn(3) then
            -- go down
            s.index = animate.down.index + t * anim_speed % animate.down.count
            s.y = s.y - speed * dt
        end
    end

    -- camera.x = s.x
    -- camera.y = s.y
    --Gravity
    --if true stop at 64
    is_in_air = s.y > ground
    if is_in_air then
        --s.y = s.y - speed * dt
    end

    -- animate slime
    for i,s in ipairs(slimes) do
        slime_update(s)
    end
end
