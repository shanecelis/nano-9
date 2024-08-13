x = 0

function _init()
  s = spr(2)
end
function _draw()
  pset(x, x, 8)
  -- pset(x, 127, 8)
  x = x + 1
  s.x = x
end
