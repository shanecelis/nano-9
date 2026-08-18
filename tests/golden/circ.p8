pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: circ / circfill
f=0
function _draw()
  cls(1)
  circfill(32, 32, 20, 8)
  circ(96, 32, 20, 11)
  circfill(64, 90, 30, 12)
  f+=1
  if f>=3 then
    extcmd("set_filename", "circ")
    extcmd("screen", 1, 1)
    extcmd("shutdown")
  end
end
