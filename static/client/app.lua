local term = require("term")
local string = require("string")
local component = require("component")
local WebSocket = require("./ws")
local event = require("event")
local internet = require("internet")
local filesystem = require("filesystem")
local thread = require("thread")
local os = require("os")
local colors = require("colors")
local RUi = require("rui")


local gpu = component.gpu;

local API_KEY_LOCATION = "/home/app/.APIKEY"
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



-- starting point of execution
function main()
    -- initialize new client if api key is not already setup
    if not filesystem.exists(API_KEY_LOCATION) then
        client_initialization()
    end

    local api_keyfile = filesystem.open(API_KEY_LOCATION);
    local api_key = api_keyfile:read(API_KEY_LEN);
    print("Successfully loaded key from file.")
    
    -- setup connection to websocket setver
    -- todo move this all to thread fn
    local ws = WebSocket.new({
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
    print("Connected to WebSocket server!")

    -- catch interrupted and disconnect websocket cleanly before exiting threads and self
    event.listen("interrupted", function()
        print("Exiting...")
        ws:close()
        running = false
        os.exit(1)
    end)

    local handle_event = thread.create(event_thread, ws);
    local handle_command = thread.create(command_thread, ws);
    -- local handle_ui = thread.create(ui_thread);

    RUi.new();

    -- Read incoming messages
    while true do
        local messageType, message, err = ws:readMessage()
        if err then return print('Websocket Error: ' .. err) end
        if messageType == WebSocket.MESSAGE_TYPES.TEXT then
            print('Message Received: ' .. message)
            messageBuffer[#messageBuffer + 1] = message
        elseif messageType == WebSocket.MESSAGE_TYPES.PING then
            ws:pong(message)
        elseif messageType == WebSocket.MESSAGE_TYPES.PONG then
            -- ignore
        end

        os.sleep(1)
    end

    thread.waitForAll({handle_event, handle_command, handle_ui})
end

function command_thread(ws_client)
    while running do
        if #messageBuffer > 0 then
            -- process message
        end
        os.sleep(1)
    end
end


--this needs to handle other events and delegate them to the other threads
function event_thread(ws_client)
    while running do
        local ev = { event.pull() };
        if ev[1] == "touch" then
            ws_client:send("HELLO!");
        end
    end
end

function ui_thread()
    while running do
        os.sleep(100);
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


    local headers = {
        ["Content-Type"] = "application/json"
    }

    local response = internet.request(SERVER_ADDRESS.."/api/authenticate", req_body, headers, "POST")
    local result = ""
    for chunk in response do result = result..chunk end
    local keyfile = filesystem.open(API_KEY_LOCATION, "w")
    keyfile:write(result)
    keyfile:close()

    return "Success!"
end

function create_json(atable)
    if table == nil or next(atable) == nil then
        return "{}"
    end
    local result = {}
    for key, value in pairs(atable) do
        -- prepare json key-value pairs and save them in separate table
        table.insert(result, string.format("\"%s\":%s", key, value))
    end
    -- get simple json string
    return "{" .. table.concat(result, ",") .. "}"

end

main()

