-- rui
local component = require("component");
local event = require("event");
local term = require("term");
local computer = require("computer");

-- todo test for gpu existance and abilities in init
local gpu = component.gpu;

local term_colours = {};
local popped_colours = {};


-- color constants
---@enum colours
local COLOURS = {

}

--- used for setting the base colour of various UI elements.
---@enum colour_set
local COLOUR_SET = {
    NORMAL_BG = 0x408f4b,

    EMPTY_FIELD = 0xbdbdbd,
    SELECTED_FIELD = 0xbdbdff,
    SUBMITTED_FIELD = 0x78dea2,
    TEXT_NORMAL = 0xFFFFFF,
    BUTTON_NORMAL = 0x6e6b68,
    BUTTON_PRESSED = 0x5e5c59,
    BUTTON_TEXT = 0xFFFFFF,
}



---@class RUi
local RUi = {}
RUi.__index = RUi;


---@enum button_sizes
RUi.BUTTON_SIZES = {
    SMALL = {6, 3},
    NORMAL = {10, 3},
    LONG = {20, 3},
    XLONG = {30, 3},
}

RUi.TEXTFIELD_SIZES = {
    SMALL = {6, 3},
    NORMAL = {10, 3},
    LONG = {20, 3},
    XLONG = {30, 3},
}


--------------------------------------------------- INIT / CLEANUP ----------------------------------------------------
function RUi.new()
    local Ui = {}
    setmetatable(Ui, RUi)
    Ui.elements = {
        buttons = {},
        labels = {},
        textfields = {},
    }
    Ui.clicked = false;
    Ui.resolution = {gpu.getResolution()};
    Ui.selected_field = nil; -- set to the currently seleted textfield/textarea (if there is one)
    
    Ui.display_buffer = gpu.allocateBuffer(Ui.resolution[1], Ui.resolution[2])
    -- enroll touch listener
    
    Ui.event_timers = {
        render = event.timer(0.2, function() Ui:render_once() end, math.huge);
    }
    
    Ui.event_listeners = {
        touch = event.listen("touch", function(...) Ui:touch_handler(...) end);
        key_down = event.listen("key_down", function(...) Ui:key_handler(...) end);
        key_up = event.listen("key_up", function(...) Ui:key_handler(...) end);
    }

    Ui.pressed_keys = {};

    Ui:push_colour(term_colours);
    gpu.setActiveBuffer(Ui.display_buffer)
    gpu.setBackground(COLOUR_SET.NORMAL_BG)
    Ui:clear()

    -- setup render loop
    return Ui;
end


-- MUST be called before program terminates to terminate cleanly, 
-- not leaving computer in unworking state.
function RUi:cleanup()
    self:pop_colour(term_colours);
    for _, listener in self.event_listeners do
        event.ignore(listener);
    end

    for _, timer in self.event_timers do
        event.cancel(timer);
    end

    gpu.setActiveBuffer(0)
end

--- clear the screen, setting the screen to the current set background colour
function RUi:clear()
    local x,y = gpu.getResolution()
    gpu.fill(1, 1, x, y, " ")
end

function RUi:fill(x2, x1, y2, y1, colour)
    local oldColour = gpu.setBackground(colour, false)
    gpu.fill(x1, y1, x2 - x1, y2 - y1, " ")
    gpu.setBackground(oldColour)
end

-----------------------------------------------------------------------------------------------------------------------


--------------------------------------------------  RENDERER  ---------------------------------------------------------
function RUi:render_once()
    for i, element_bundle in pairs(self.elements) do
        for j, element in ipairs(element_bundle) do
            element:render();
        end
    end
    
    gpu.bitblt(0, 1, 1, self.resolution[1], self.resolution[2], self.display_buffer)
end
-----------------------------------------------------------------------------------------------------------------------


------------------------------------------------ BASIC SHAPES ---------------------------------------------------------
function RUi:rect(x, y, w, l, colour)
    self:push_colour()
    gpu.setBackground(colour, false)
    gpu.fill(x, y, w*2, l, " ") -- *2 here to make up for the weird aspect ratio
    self:pop_colour()
end

function RUi:square(x, y, w, colour)
    self:push_colour()
    gpu.setBackground(colour, false)
    gpu.fill(x, y, w*2, w, " ") -- *2 here to make up for the weird aspect ratio
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

-----------------------------------------------------------------------------------------------------------------------


---------------------------------------------------- GUI ELEMENTS -----------------------------------------------------

------ Label ------
function RUi:label(label, x, y)
    local label = {};
    label.ui = self;
    label.fg = COLOUR_SET.TEXT_NORMAL;
    self:push_colour()
    
    self:pop_colour()

    function label:render()
        self.ui:push_colour()
        gpu.setForeground(self.fg)
        gpu.set(x, y, label);
        self.ui:pop_colour()
    end

    return label;
end


------ Button -----
function RUi:button(label, x, y, size, callback)
    local button = {};
    button.ui = self;
    button.label = label;
    button.pos = {x = x, y = y};
    button.callback = callback;
    button.size = size or self.BUTTON_SIZES.NORMAL;
    
    button.bg = COLOUR_SET.BUTTON_NORMAL;
    button.fg = COLOUR_SET.BUTTON_TEXT;

    -- need methods to remove button after it is created
    function button:render()
        self.ui:push_colour()
        gpu.setBackground(self.bg);
        gpu.setForeground(self.fg);
        gpu.fill(self.pos.x, self.pos.y, self.size[1], self.size[2], " ");
        gpu.set((self.pos.x+(self.size[1]/2)) - ((#self.label)/2), self.pos.y+1, self.label);
        self.ui:pop_colour()
    end

    table.insert(self.elements.buttons, button)
    return button;
end

-- may move this into touch handler
function RUi:generic_button_handler(button)
    -- Change button color to indicate it's pressed
    button.bg = COLOUR_SET.BUTTON_PRESSED

    local function on_button_release(_, _, button_id)
        -- Reset button color when released
        button.bg = COLOUR_SET.BUTTON_NORMAL
        
        -- Unregister the event handler
        event.cancel(button_release_id)

        -- Make the beep
        computer.beep(500, 0.02)

        -- Execute the button's callback if defined
        if button.callback then
            button.callback()
        end
    end

    -- Register the event handler for button release
    button_release_id = event.listen("drop", on_button_release)
end
----------------

--- Textfield --
function RUi:textfield(placeholder, x, y, callback)
    local textfield = {};
    textfield.ui = self;
    textfield.pos = {x = x, y = y};

    textfield.buffer = "";
    textfield.placeholder = placeholder;
    textfield.selected = false;
    textfield.size = self.TEXTFIELD_SIZES.NORMAL;
    textfield.window = "";
    textfield.window_size = textfield.size[1] - 2;
    textfield.callback = callback;

    textfield.bg = COLOUR_SET.EMPTY_FIELD;
    textfield.fg = COLOUR_SET.TEXT_NORMAL;

    function textfield:render()
        self.ui:push_colour()
        gpu.setBackground(self.bg);
        gpu.setForeground(self.fg);
        
        gpu.fill(self.pos.x, self.pos.y, self.size[1], self.size[2], " ")
        gpu.set(self.pos.x + 1, self.pos.y + 1, self.window);
        self.ui:pop_colour()
    end

    function textfield:set_selected(selected)
        textfield.selected = selected;
        if selected then
            self.ui.selected_field = self;
            textfield.bg = COLOUR_SET.SELECTED_FIELD;
        else
            self.ui.selected_field = nil;
            textfield.bg = COLOUR_SET.EMPTY_FIELD;
        end
    end

    function textfield:update_window()
        if self.window_size >= #self.buffer then
            self.window = self.buffer;
        else
            self.window = self.buffer:sub(-self.window_size);
        end
    end

    table.insert(self.elements.textfields, textfield);
    return textfield;
end


function RUi:terminal()
    local terminal = {}
end



-- -- Save the current colors (background and foreground) for later use
function RUi:push_colour(set)
    local cache_location = set or popped_colours;
    -- Check if there are any colors already pushed (if so, it should be empty)
    assert(#cache_location == 0, "push_colour before pop_colour")

    -- Save current background and foreground colors
    local bg_color, is_bg_palette = gpu.getBackground()
    local fg_color, is_fg_palette = gpu.getForeground()

    -- Store the colors in the popped_colours table
    table.insert(cache_location, {bg_color, is_bg_palette})
    table.insert(cache_location, {fg_color, is_fg_palette})
end

-- -- Restore the previously saved colors
function RUi:pop_colour(set)
    local cache_location = set or popped_colours;
    -- Check if we have the colors saved to pop
    assert(#cache_location == 2, "pop_colour before push_colour")

    -- Retrieve the saved colors from the popped_colours table
    local old_bg = table.remove(cache_location)
    local old_fg = table.remove(cache_location)

    -- Apply the saved colors
    gpu.setBackground(old_bg[1], old_bg[2])
    gpu.setForeground(old_fg[1], old_fg[2])
end


function RUi:touch_handler(_, scr_address, x, y, button, player)
    self.clicked = true;

    -- check if touch is on a textfield
    for index, textfield in ipairs(self.elements.textfields) do
        local w, h = table.unpack(textfield.size);
        if check_if_within({textfield.pos.x, textfield.pos.y, w, h}, {x=x, y=y}) then
            textfield:set_selected(true)
            return;
        end
    end

    -- if it wasn't on a textfield, but one was selected, it can be unselected now.
    if self.selected_field then
        self.selected_field:set_selected(false);
    end

    -- check if touch is on a button, if it is, call the handler.
    for index, button in ipairs(self.elements.buttons) do
        local w, h = table.unpack(button.size);

        if check_if_within({button.pos.x, button.pos.y, w, h}, {x=x, y=y}) then
            self:generic_button_handler(button)
            return;
        end
    end
end

function RUi:key_handler(event_type, keyboard_address, char, code, playerName)
    if event_type == "key_down" then
        -- some input other than ascii
        if char == 0 then
            table.insert(self.pressed_keys, code);
            return;
        end

        if self.selected_field then
            if code == 14 then -- backspace
                if #self.selected_field.buffer > 0 then
                    self.selected_field.buffer = self.selected_field.buffer:sub(1, -2); -- remove last element
                    self.selected_field:update_window();
                end
                return;
            elseif code == 28 then -- enter
                if self.selected_field.callback then
                    self.selected_field.callback(self.selected_field.buffer)
                end
                self.selected_field.buffer = "";
                self.selected_field.bg = COLOUR_SET.SUBMITTED_FIELD;
                self.selected_field:update_window();
                return;
            end

            local letter = get_letter_from_char(char);
            if #self.pressed_keys > 0 then
                for _, key in ipairs(self.pressed_keys) do
                    if key == 42 then -- shift
                        char = char -42;
                    end
                end
                -- if its shift, we want to -42 from the char
            end
            
            if letter ~= "-1" then
                self.selected_field.buffer = self.selected_field.buffer .. letter;
                self.selected_field:update_window();
            end
        end
    
    else -- key released
        for i, key in self.pressed_keys do
            if key == code then
                table.remove(self.pressed_keys, i);
            end
        end
    end
end


-- takes in data like {0, 0, 0, 0}, {x=0, y=0}
function check_if_within(rect, points)
    local x = points.x;
    local y = points.y;
    local rec_x = rect[1];
    local rec_y = rect[2];
    local rec_w = rect[3];
    local rec_h = rect[4];

    return x >= rec_x       and 
           x <= rec_x+rec_w and 
           y >= rec_y       and 
           y <= rec_y+rec_h;
end


-- Function to get the character from an ASCII code


local keyboard_map = {
    [32] = ' ', [33] = '!', [34] = '"', [35] = '#', [36] = '$', [37] = '%', [38] = '&',
    [39] = "'", [40] = '(', [41] = ')', [42] = '*', [43] = '+', [44] = ',', [45] = '-',
    [46] = '.', [47] = '/', [48] = '0', [49] = '1', [50] = '2', [51] = '3', [52] = '4',
    [53] = '5', [54] = '6', [55] = '7', [56] = '8', [57] = '9', [58] = ':', [59] = ';',
    [60] = '<', [61] = '=', [62] = '>', [63] = '?', [64] = '@', [65] = 'A', [66] = 'B',
    [67] = 'C', [68] = 'D', [69] = 'E', [70] = 'F', [71] = 'G', [72] = 'H', [73] = 'I',
    [74] = 'J', [75] = 'K', [76] = 'L', [77] = 'M', [78] = 'N', [79] = 'O', [80] = 'P',
    [81] = 'Q', [82] = 'R', [83] = 'S', [84] = 'T', [85] = 'U', [86] = 'V', [87] = 'W',
    [88] = 'X', [89] = 'Y', [90] = 'Z', [91] = '[', [92] = '\\', [93] = ']', [94] = '^',
    [95] = '_', [96] = '`', [97] = 'a', [98] = 'b', [99] = 'c', [100] = 'd', [101] = 'e',
    [102] = 'f', [103] = 'g', [104] = 'h', [105] = 'i', [106] = 'j', [107] = 'k', [108] = 'l',
    [109] = 'm', [110] = 'n', [111] = 'o', [112] = 'p', [113] = 'q', [114] = 'r', [115] = 's',
    [116] = 't', [117] = 'u', [118] = 'v', [119] = 'w', [120] = 'x', [121] = 'y', [122] = 'z',
    [123] = '{', [124] = '|', [125] = '}', [126] = '~'
}

function get_letter_from_char(code)
    return keyboard_map[code] or -1
end
  



return RUi;

