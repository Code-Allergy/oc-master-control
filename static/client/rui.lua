-- rui


-- color constants
---@enum colours
local COLOURS = {

}

---@enum colour_set
local COLOUR_SET = {
    TEXT_NORMAL = 0xFFFFFF,
    BUTTON_NORMAL = 0x6e6b68,
    BUTTON_PRESSED = 0x5e5c59,
    BUTTON_TEXT = 0xFFFFFF,
}



local component = require("component");
local event = require("event")

-- test for this instead
local gpu = component.gpu;

local popped_colours = {};


---@class RUi
---@field 
---@field 
---@field 
local RUi = {}
RUi.__index = RUi;


---@enum button_sizes
RUi.BUTTON_SIZES = {
    SMALL = {6, 3},
    NORMAL = {10, 3},
    LONG = {20, 3},
    XLONG = {30, 3},
}



function RUi.new()
    local Ui = {}
    setmetatable(Ui, RUi)
    Ui.components = {
        buttons = {},
        labels = {},
        textFields = {},
    }

    Ui.display_thread = nil;

    -- enroll touch listener
    event.listen("touch", function(...) Ui:touch_handler(...) end);

    -- setup render loop
    return Ui;
end

function RUi:rect(x, y, w, l, colour)
    self:push_colour()
    gpu.setBackground(colour, false)
    gpu.fill(x, y, w*2, l, " ")
    self:pop_colour()
end

function RUi:square(x, y, w, colour)
    self:push_colour()
    gpu.setBackground(colour, false)
    gpu.fill(x, y, w*2, w, " ")
    self:pop_colour()
end

-- TODO maybe
function RUi:ellipse(x, y, xr, yr, colour)
    error("unimplemented")
end
-- TODO maybe
function RUi:sphere(x, y, r, colour)
    error("unimplemented")
end

function RUi:clear()
    local x,y = gpu.getResolution()
    gpu.fill(1, 1, x, y, " ")
end

-- --BUTTON_SIZES.NORMAL

function RUi:fill(x2, x1, y2, y1, colour)
    local oldColour = gpu.setBackground(colour, false)
    gpu.fill(x1, y1, x2 - x1, y2 - y1, " ")
    gpu.setBackground(oldColour)
end

function RUi:button(label, x, y, size, callback)
    local button = {};
    button.label = label;
    button.pos = {x, y};
    button.callback = callback;
    button.size = size or self.BUTTON_SIZES.NORMAL;

    -- need methods to remove button after it is created

    self:push_colour()
    gpu.setBackground(COLOUR_SET.BUTTON_NORMAL);
    gpu.setForeground(COLOUR_SET.BUTTON_TEXT);
    gpu.fill(x, y, button.size[1], button.size[2], " ");
    gpu.set((x+(button.size[1]/2)) - ((#label)/2), y+1, label);
    self:pop_colour()

    table.insert(self.components.buttons, button)
    return button;
end

-- commands to run before calling final button action callback (ex - change colour, play beep sound)
-- call this within the touch handler thread
function RUi:generic_button_handler(callback)
    print("Callback!")
    if callback ~= nil then
        print("Callback called!")
        callback()
    end
end

-- -- Save the current colors (background and foreground) for later use
function RUi:push_colour()
    -- Check if there are any colors already pushed (if so, it should be empty)
    assert(#popped_colours == 0, "push_colour before pop_colour")

    -- Save current background and foreground colors
    local bg_color, is_bg_palette = gpu.getBackground()
    local fg_color, is_fg_palette = gpu.getForeground()

    -- Store the colors in the popped_colours table
    table.insert(popped_colours, {bg_color, is_bg_palette})
    table.insert(popped_colours, {fg_color, is_fg_palette})
end

-- -- Restore the previously saved colors
function RUi:pop_colour()
    -- Check if we have the colors saved to pop
    assert(#popped_colours == 2, "pop_colour before push_colour")

    -- Retrieve the saved colors from the popped_colours table
    local old_bg = table.remove(popped_colours)
    local old_fg = table.remove(popped_colours)

    -- Apply the saved colors
    gpu.setBackground(old_bg[1], old_bg[2])
    gpu.setForeground(old_fg[1], old_fg[2])
end



function RUi:touch_handler(_, scr_address, x, y, button, player)
    -- check if touch is on a button, if it is, call the handler.
    print("In touch handler! #buttons: " .. tostring(self.components.buttons))
    for index, button in ipairs(self.components.buttons) do
        local btn_x, btn_y = table.unpack(button.pos);
        local w, h = table.unpack(button.size);

        if x >= btn_x and x <= btn_x+w and y >= btn_y and y <= btn_y+h then
            self:generic_button_handler(button.callback)
        end
    end
end


return RUi;

