pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: cls white
function _draw()
  cls(7)
  extcmd("set_filename", "cls-white")
  extcmd("screen", 1, 1)
  extcmd("shutdown")
end
