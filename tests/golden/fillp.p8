pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: fillp dither
function _draw()
  cls(0)
  fillp(0x5a5a)
  rectfill(10, 10, 70, 70, 8)
  fillp()
  rectfill(80, 10, 120, 50, 11)
  extcmd("set_filename", "fillp")
  extcmd("screen", 1, 1)
  extcmd("shutdown")
end
