pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: oval / ovalfill
function _draw()
  cls(1)
  ovalfill(10, 20, 50, 60, 8)
  oval(70, 20, 120, 80, 11)
  ovalfill(30, 70, 90, 120, 12)
  if stat(6)=="headless" then
    extcmd("set_filename", "oval")
    extcmd("screen", 1, 1)
    extcmd("shutdown")
  end
end
