pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: cls
function _draw()
  cls(8)
  extcmd("set_filename", "cls")
  extcmd("screen", 1, 1)
  extcmd("shutdown")
end
