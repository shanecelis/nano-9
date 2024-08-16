function table_remove(t, fn)
  local j = 1
  for i=1,#t do
    if fn(t[i]) then
      -- toss this one
      t[i] = nil
    else
      -- keep this one
      if i ~= j then
          t[j], t[i] = t[i], nil
      end
      j = j + 1
    end
  end
end

function table_remove_eq(t, e)
    table_remove(t, function(x)
        return x == e
    end)
end
