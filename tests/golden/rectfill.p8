pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: rectfill / rect
function _draw()
  cls(1)
  rectfill(10, 10, 50, 40, 8)
  rect(60, 20, 110, 70, 11)
  rectfill(20, 80, 100, 110, 12)
  extcmd("set_filename", "rectfill")
  extcmd("screen", 1, 1)
  extcmd("shutdown")
end
