-- text:print("Hello, World!")

function _init()
    background.image:set_pixel(16,32, 8)
    pixie = image:load("images/pixie.png")
    jar = pixie:spr(64,64)


    -- sprite = nil
    -- sprite.x = 64
    -- sprite.y = 64
    -- pixie:spr()
end

x = 0
c = {r = 0, g = 0, b = 1}
function _update()
    background.image:set_pixel(x, x, c)
    x = x + 1
end
