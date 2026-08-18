pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: color / default pen
f=0
function _draw()
  cls(1)
  color(8)
  rectfill(10, 10, 40, 40)
  color(11)
  pset(50, 50)
  pset(51, 50)
  pset(52, 50)
  color(12)
  rect(60, 20, 110, 70)
  f+=1
  if f>=3 then
    extcmd("set_filename", "color")
    extcmd("screen", 1, 1)
    extcmd("shutdown")
  end
end
