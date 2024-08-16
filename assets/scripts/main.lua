-- main.lua
--
-- A great cat game
require("assets/scripts/actor")
require("assets/scripts/slime")

speed = 30
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
    },
    hit = {
        index = 7,
        count = 4,
    },
    bounce = {
        index = 14,
        count = 7,
    }
}
ground = -64


-- _init() is called once per load.
function _init()
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

    -- load slime
    slime_image = loadimg("images/slime/slime.png")
    slime_image:set_grid(16,16,7,3)

    s1 = slime_image:spr(0)
    s1.y = ground
    s1.anchor = { 0, -1 }

    s2 = actor:new {
        sprite = slime_image:spr(0),
        anims = {
            idle = {
                ticks = 8,
                frames = {0,1,2,3,4,5},
                -- index = 0,
                -- count = 6,
            },
            hit = {
                ticks = 3,
                frames = {7,8,9,10},
                -- index = 7,
                -- count = 4,
            },
            bounce = {
                ticks = 3,
                frames = {14,15,16,17,18,19,20},
                -- index = 14,
                -- count = 7,
            }
        }
    }
    s2.sprite.x = 22
    s2.sprite.y = ground
    s2.sprite.anchor = { 0, -1 }
    s2:set_anim("idle")

    s3 = slime:new {}
    s3.sprite.x = 44
    s3.sprite.y = ground

    -- clear the screen
    cls(1)
end

function _draw()
    s2:draw()
    s3:draw()

end

-- _draw() is called once per frame, or 60 times a second.
function _update()
    s2:update()
    s3:update()
    -- t is time in seconds.
    t = time()
    -- anim_speed is our animation speed. How many frames per second?
    anim_speed = 8
    -- dt is the time since we last were here. Usually 1/60th of a second.
    dt = delta_time()

    -- Check if 'z' is pressed.
    if btnp(4) then
        scratch_start = t -- note cat scratch start time.
        distance = math.abs( s.x -s1.sprite.x )
        print("hit distance ", distance)
        if distance <= 17.5 then
            -- s1.color = 8
        else
            -- s1.color = nil
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
    s1.index = t * anim_speed % 6
end
