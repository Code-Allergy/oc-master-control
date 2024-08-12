-- rui


local component = require("component");
local colors = require("colors")

-- test for this instead
local gpu = component.gpu;


local popped_colours = {}


---@class RUi
---@field 
---@field 
---@field 
local RUi = {}
RUi.__index = RUi;

function RUi.new()
    local Ui = {}
    setmetatable(Ui, RUi)

    fill(3, 3, 3, 3, colors.gray)
    fill(8, 8, 8, 8, colors.pink)
    fill(11, 3, 2, 5, colors.gray)


    return Ui;
end


function RUi:rect(x, y, w, l, colour)
    gpu.fill(x, y, w, l)
end

function RUi:square(x, y, w, l, colour)

end

function RUi:ellipse(x, y, xr, yr, colour)

end

function RUi:sphere(x, y, r, colour)

end

function RUi:clear()
    x,y = gpu.getResolution()
    gpu.fill(1, 1, x, y, " ")
end


function fill(x2, x1, y2, y1, colour)
    push_colour()
    gpu.setForeground(colour)   
    gpu.fill(x1, y1, x2 - x1, y2 - y1, " ")
    pop_colour()
end

-- Save the current colors (background and foreground) for later use
function push_colour()
    -- Check if there are any colors already pushed (if so, it should be empty)
    assert(#popped_colours == 0, "push_colour before pop_colour")

    -- Save current background and foreground colors
    local bg_color, is_bg_palette = gpu.getBackground()
    local fg_color, is_fg_palette = gpu.getForeground()

    -- Store the colors in the popped_colours table
    table.insert(popped_colours, {bg_color, is_bg_palette})
    table.insert(popped_colours, {fg_color, is_fg_palette})
end

-- Restore the previously saved colors
function pop_colour()
    -- Check if we have the colors saved to pop
    assert(#popped_colours == 2, "pop_colour before push_colour")

    -- Retrieve the saved colors from the popped_colours table
    local old_bg = table.remove(popped_colours)
    local old_fg = table.remove(popped_colours)

    -- Apply the saved colors
    gpu.setBackground(old_bg[1], old_bg[2])
    gpu.setForeground(old_fg[1], old_fg[2])
end


return RUi
