
actor = {
    sprite = nil, -- start sprite
    anim = {}, -- animation values
}

-- make an actor
function actor:new(a, sprite)
    a = a or {}
    setmetatable(a, self)
    self.sprite = sprite or self.sprite
    -- self.__index = sprite
    self.__index = self
    -- self.__newindex = sprite
    -- function(table, key)
    --   -- return self[key] or sprite[key]
    --   return sprite[key]
    -- end
    return a
end

function actor:set_anim(anim)
    if anim==self.curanim then return end --early out.
    self.animtick=self.anims[anim].ticks--ticks count down.
    self.curanim=anim
    self.curframe=1
end

function actor:draw()
    if not(self.curanim) then return end --early out.
    local a=self.anims[self.curanim]
    local frame=a.frames[self.curframe]
    self.sprite.index = frame
end

function actor:update()
    if not(self.curanim) then return end --early out.
    self.animtick = self.animtick - 1
    if self.animtick<=0 then
        local a=self.anims[self.curanim]
        self.curframe=self.curframe % #a.frames + 1
        self.animtick=a.ticks -- init timer
    end
end
