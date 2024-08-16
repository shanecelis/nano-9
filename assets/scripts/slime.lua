slime = actor:new {
    anims = {
        idle = {
            ticks = 8,
            frames = {0,1,2,3,4,5},
            -- index = 0,
            -- count = 6,
        },
        hit = {
            ticks = 3,
            frames = {7,8,9,10},
            -- index = 7,
            -- count = 4,
        },
        bounce = {
            ticks = 3,
            frames = {14,15,16,17,18,19,20},
            -- index = 14,
            -- count = 7,
        }
    },
    new = function(self, o)
        o = actor.new(self, o)
        o.sprite = slime_image:spr(0)
        o:set_anim("idle")
        o.sprite.anchor = { 0, -1 }
        return o
    end
}
