pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: oval / ovalfill small
function _draw()
  cls(0)
  ovalfill(8, 8, 8, 8, 8)
  ovalfill(16, 8, 20, 12, 8)
  ovalfill(28, 8, 40, 16, 8)
  ovalfill(48, 8, 70, 20, 8)
  oval(8, 32, 8, 32, 11)
  oval(16, 32, 20, 36, 11)
  oval(28, 32, 40, 40, 11)
  oval(48, 32, 70, 44, 11)
  ovalfill(9, 56, 21, 64, 12)
  oval(25, 56, 37, 64, 12)
  if stat(6)=="headless" then
    extcmd("set_filename", "oval-small")
    extcmd("screen", 1, 1)
    extcmd("shutdown")
  end
end
