pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: circ / circfill
function _draw()
  cls(1)
  circfill(32, 32, 20, 8)
  circ(96, 32, 20, 11)
  circfill(64, 90, 30, 12)
  extcmd("set_filename", "circ")
  extcmd("screen", 1, 1)
  extcmd("shutdown")
end
