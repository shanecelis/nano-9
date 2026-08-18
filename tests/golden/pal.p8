pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: pal remap
f=0
function _draw()
  cls(0)
  pal(8, 12)
  rectfill(8, 8, 56, 56, 8)
  pal()
  rectfill(72, 8, 120, 56, 8)
  pal(8, 11)
  pset(10, 80, 8)
  pset(11, 80, 8)
  pset(12, 80, 8)
  pal()
  pset(20, 80, 8)
  f+=1
  if f>=3 then
    extcmd("set_filename", "pal")
    extcmd("screen", 1, 1)
    extcmd("shutdown")
  end
end
