pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: line
function _draw()
  cls(0)
  line(0, 0, 127, 127, 7)
  line(0, 1, 126, 127, 7)
  line(0, 127, 127, 0, 8)
  line(10, 64, 117, 64, 11)
  line(64, 10, 64, 117, 12)
  extcmd("set_filename", "line")
  extcmd("screen", 1, 1)
  extcmd("shutdown")
end
