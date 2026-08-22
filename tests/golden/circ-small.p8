pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: circ / circfill small radii
function _draw()
  cls(0)
  circfill(8, 8, 0, 8)
  circfill(16, 8, 1, 8)
  circfill(28, 8, 2, 8)
  circfill(44, 8, 3, 8)
  circfill(64, 8, 4, 8)
  circ(8, 32, 0, 11)
  circ(16, 32, 1, 11)
  circ(28, 32, 2, 11)
  circ(44, 32, 3, 11)
  circ(64, 32, 4, 11)
  circfill(9, 56, 2, 12)
  circ(21, 56, 2, 12)
  extcmd("set_filename", "circ-small")
  extcmd("screen", 1, 1)
  extcmd("shutdown")
end
