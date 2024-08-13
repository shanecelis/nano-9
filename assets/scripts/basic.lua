x = 0
speed = 2
animate ={
  right = {
    index = 8,
    count = 4
  },
}

function _init()
  -- s = spr(8 * 24 + 15)
  s = spr(8)
  s.sx = 32 * 2
  s.sy = 32 * 2
  -- s.sy = 24
  -- s.color = 15
  -- s:drop()
  -- s = nil
  t = 0
end

function _draw()
  pset(x, x, 8)
  -- pset(x, 127, 8)
  x = x + 1
  t = time() * speed * 4
  -- t = t + 0.5
  if btn(0) then
    s.index = animate.right.index + t % animate.right.count
    s.x = s.x - speed
    s.flip_x = true
  elseif btn(1) then
    s.index = animate.right.index + t % animate.right.count
    s.x = s.x + speed
    s.flip_x = false
  elseif btn(2) then
    s.y = s.y - speed
  elseif btn(3) then
    s.y = s.y + speed
  end
  -- s.x = x
  -- s.y = x
  -- s.index = x
end
