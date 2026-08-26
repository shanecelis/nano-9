pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: camera offset
function _draw()
  cls(1)
  rectfill(0, 0, 20, 20, 8)
  camera(40, 20)
  rectfill(0, 0, 20, 20, 11)
  camera(0, 0)
  rectfill(100, 100, 120, 120, 12)
  if stat(6)=="headless" then
    extcmd("set_filename", "camera")
    extcmd("screen", 1, 1)
    extcmd("shutdown")
  end
end
