pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: print cursor / newline / default pen
function _draw()
  cls(0)
  -- default pen (6)
  print("one", 0, 0)
  print("two", 0, 6)
  -- cursor sets position for the next print
  cursor(8, 24)
  print("cur")
  cursor(8, 30)
  print("next")
  print("red", 8, 48, 8)
  print("fol", 8, 54, 8)
  print("a\nb", 64, 8, 11)
  print(123, 64, 32, 12)
  -- cursor sets pen color
  cursor(8, 80, 8)
  print("pen")
  if stat(6)=="headless" then
    extcmd("set_filename", "print-cursor")
    extcmd("screen", 1, 1)
    extcmd("shutdown")
  end
end
