pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: line points / axis / reverse
function _draw()
  cls(0)
  line(8, 8, 8, 8, 8)
  line(10, 8, 30, 8, 7)
  line(30, 12, 10, 12, 11)
  line(8, 16, 8, 36, 8)
  line(12, 36, 12, 16, 11)
  line(40, 8, 80, 10, 12)
  line(40, 16, 42, 50, 12)
  line(50, 40, 90, 40, 8)
  line(64, 20, 64, 60, 11)
  extcmd("set_filename", "line-axis")
  extcmd("screen", 1, 1)
  extcmd("shutdown")
end
