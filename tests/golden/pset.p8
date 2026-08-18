pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: pset
f=0
function _draw()
  cls(1)
  pset(10, 10, 7)
  pset(20, 30, 8)
  pset(64, 64, 12)
  pset(127, 127, 11)
  f+=1
  if f>=3 then
    extcmd("set_filename", "pset")
    extcmd("screen", 1, 1)
    extcmd("shutdown")
  end
end
