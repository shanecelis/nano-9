pico-8 cartridge // http://www.pico-8.com
version 42
__lua__
-- golden: print
f=0
function _draw()
  cls(0)
  print("hello", 8, 8, 7)
  print("pico-8", 8, 20, 11)
  print("123", 8, 32, 8)
  f+=1
  if f>=3 then
    extcmd("set_filename", "print")
    extcmd("screen", 1, 1)
    extcmd("shutdown")
  end
end
