
local Util = {}

function Util.split_on_newl(str)
    local result = {}
    for line in string.gmatch(str, "[^\r\n]+") do
        table.insert(result, line)
    end
    return result
end

-- TODO this only works with two items
function Util.split_on_space(str)
    local spacePos = string.find(str, " ")
    if spacePos then
        local part1 = string.sub(str, 1, spacePos - 1)
        local part2 = string.sub(str, spacePos + 1)
        return part1, part2
    else
        -- No space found; return the original string as the first part and an empty string as the second part
        return str, ""
    end
end

return Util;
