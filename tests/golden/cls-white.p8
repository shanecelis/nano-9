pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: cls white
f=0
function _draw()
  cls(7)
  f+=1
  if f>=3 then
    extcmd("set_filename", "cls-white")
    extcmd("screen", 1, 1)
    extcmd("shutdown")
  end
end
