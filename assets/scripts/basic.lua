x = 0

function _init()
  s = spr(8 * 24 + 15)
  -- s.color = 15
  -- s:drop()
  -- s = nil
end

function _draw()
  pset(x, x, 8)
  -- pset(x, 127, 8)
  x = x + 1
  if btn(0) then
    s.x = s.x - 1
    s.flip_x = false
  elseif btn(1) then
    s.x = s.x + 1
    s.flip_x = true
  elseif btn(2) then
    s.y = s.y - 1
  elseif btn(3) then
    s.y = s.y + 1
  end
  -- s.x = x
  -- s.y = x
  -- s.index = x
end
