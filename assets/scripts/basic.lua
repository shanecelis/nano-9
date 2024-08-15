-- local other = require("other")
-- setpal(img)
x = 0
speed = 2
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

-- _init() is called once per load.
function _init()
  x = 0
  -- s = spr(8 * 24 + 15)
  --
  s = spr(8) -- load cat sprite
  --s.color = 1
  -- Make sprite two times bigger.
  s.sx = 2 * 32
  s.sy = 2 * 32
  t = 0
  hearts = loadimg("images/heart-of-a-thousand-miles.png");
  hearts:set_grid(12, 12, 10, 10)
  h1 = hearts:spr(0)
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

    -- t is our animation time.
    t = time() * speed * 4

    -- Check if 'z' is pressed.
    if btnp(4) then
        scratch_start = t -- note cat scratch start time.
    end

    if btnp(5) then
        jump_start = t -- note cat scratch start time.
        s.y = s.y - 188
    end

    if scratch_start then
        -- cat is scratching

        -- scratch_frame is either 0, 1, 2 for the three frames.
        scratch_frame = t - scratch_start
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
            s.index = animate.right.index + t % animate.right.count
            s.x = s.x - speed
            s.flip_x = true
        elseif btn(1) then
            -- go right
            s.index = animate.right.index + t % animate.right.count
            s.x = s.x + speed
            s.flip_x = false
            --elseif btn(2) then
            -- go up
            s.y = s.y - speed
            --elseif btn(3) then
            -- go down
            s.y = s.y + speed
        end
    end

    --Gravity
    --if true stop at 64
    is_in_air = s.y < 256 - 32
    if is_in_air then
        s.y = s.y + speed
    end
end
