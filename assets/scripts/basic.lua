-- local other = require("other")
-- setpal(img)
x = 0
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
        count = 1
    }

}
ground = 64

-- _init() is called once per load.
function _init()
  cat_sprites = loadimg("images/Cat_Sprite.png")
  cat_sprites:set_grid(32, 32, 4, 8)
  x = 0
  s = cat_sprites:spr(8) -- load cat sprite
  s.anchor = {0, -1}   -- move the cat sprite by its bottom-center.
  --s.color = 1
  -- Make sprite two times bigger.
  -- s.sx = 2 * 32
  -- s.sy = 2 * 32
  t = 0
  hearts = loadimg("images/heart-of-a-thousand-miles.png");
  hearts:set_grid(12, 12, 10, 10)
  h1 = hearts:spr(0)
  s.anchor = {-1, 1}   -- move the heart sprite by its top-left.
  img = loadimg("images/Goblin22.png")
 -- setpal(img)
  cls(1)

end

-- _draw() is called once per frame, or 60 times a second.
function _draw()
  -- cls(1)
    -- pset(x, y, c) sets the pixel at location (x, y) to color c.
    pset(x, x, 0)
    x = x + 1

    -- t is time in seconds.
    t = time()
    -- anim_speed is our animation speed. How many frames per second?
    anim_speed = 8
    -- dt is the time since we last were here. Usually 1/60th of a second.
    dt = delta()


    -- Check if 'z' is pressed.
    if btnp(4) then
        scratch_start = t -- note cat scratch start time.
    end

    if btnp(5) then
        jump_start = t -- note cat scratch start time.
        s.y = s.y - 20
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
    is_in_air = s.y < ground - 42
    if is_in_air then
        -- s.y = s.y + speed
    end
end
