local term = require("term");
local string = require("string");
local component = require("component");
local WebSocket = require("ws");
local event = require("event");
local internet = require("internet");
local filesystem = require("filesystem");
local thread = require("thread");
local os = require("os");
local colors = require("colors");
local RUi = require("rui");
local commands = require("commands");

local gpu = component.gpu;

local API_KEY_LOCATION = "/home/.APIKEY"
local AUTH_KEY_LEN = 8
local API_KEY_LEN = 48
local SERVER_ADDRESS = "http://192.168.1.124:3000"
local WS_ADDRESS = "ws://192.168.1.124"
local WS_PORT = 3000
local WS_PATH = "/api/ws"

-- pull off top of buffer (end), and insert at start
local messageBuffer = {};

-- simple disable all threads at once
local running = true;

-- ws client
local ws = nil;


-- starting point of execution
function main()
    print("Starting up!");
    local rui = RUi.new();
    
    -- initialize new client if api key is not already setup
    if not filesystem.exists(API_KEY_LOCATION) then
        client_initialization()
    end
    
    local api_keyfile = filesystem.open(API_KEY_LOCATION);
    local api_key = api_keyfile:read(API_KEY_LEN);
    print("Keyfile was found and read successfully!");

    -- setup connection to websocket server
    -- todo move this all to thread fn
    ws = WebSocket.new({
        address = WS_ADDRESS,
        port = WS_PORT,
        path = WS_PATH,
        headers = "X-API-Key: " .. api_key
    })
    
    while true do
        local connected, err = ws:finishConnect()
        if connected then break end
        if err then return print('Failed to connect: ' .. err) end
        os.sleep(1);
    end

    print("Websocket connected!");
    
    -- catch interrupted and disconnect websocket cleanly before exiting threads and self
    event.listen("interrupted", function()
        rui:cleanup()
        ws:close()
        while ws:isOpen() do os.sleep(1) end
        print("Exiting due to interruption...")
        running = false
        term.clear()
        os.exit(1)
    end)
    
    display_layout(rui, ws);
    local handle_command = thread.create(command_thread, ws);
    -- Read incoming messages
    -- TODO solution to os.sleep problem
    while true do
        local messageType, message, err = ws:readMessage()
        if err then return print('Websocket Error: ' .. err) end
        if messageType == WebSocket.MESSAGE_TYPES.TEXT then
            print("Command received: "..message)
            messageBuffer[#messageBuffer + 1] = message
        elseif messageType == WebSocket.MESSAGE_TYPES.PING then
            ws:pong(message)
        elseif messageType == WebSocket.MESSAGE_TYPES.PONG then
            -- ignore
        end

        os.sleep(1);
    end

    thread.waitForAll({handle_command})
end

function display_layout(rui, ws)
    local button1 = rui:button("test", 26, 8, rui.BUTTON_SIZES.NORMAL, (function() ws:send("Log button pressed") end))
    local textfield = rui:textfield("TEXT", 15, 8, (function(contents) ws:send("Log " .. contents) end));
end

function command_thread(ws_client)
    while running do
        if #messageBuffer > 0 then
            local message = messageBuffer[1];
            
            local command_fn = commands[message];
            if command_fn then
                command_fn()
            end
            
            table.remove(messageBuffer, 1); 
        end
        os.sleep(1)
    end
end

function client_initialization()
    print("Create an authorization key and enter it here: ")
    result = term.read()
    
    if #result ~= AUTH_KEY_LEN+1 then
        print("Authorization key shorter than expected")
        -- goto loop
    end
    auth_code = string.sub(result, 1, -2)
    print("Authorization key entered: "..auth_code)
    print("Sending request")
    local req_body = string.format([[{"authcode": "%s"}]], auth_code)

    local response = internet.request(SERVER_ADDRESS.."/api/authenticate", 
        req_body, {["Content-Type"] = "application/json"}, "POST");
    
    local result = ""
    for chunk in response do result = result..chunk end
    local keyfile = filesystem.open(API_KEY_LOCATION, "w")
    keyfile:write(result)
    keyfile:close()

    return "Successfully connected to server and obtained an API key!"
end

main()

