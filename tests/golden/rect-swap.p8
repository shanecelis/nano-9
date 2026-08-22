pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: rect / rectfill swapped corners
function _draw()
  cls(1)
  rectfill(40, 30, 10, 10, 8)
  rectfill(10, 50, 40, 70, 11)
  rect(90, 40, 50, 10, 12)
  rect(50, 50, 90, 80, 7)
  extcmd("set_filename", "rect-swap")
  extcmd("screen", 1, 1)
  extcmd("shutdown")
end
