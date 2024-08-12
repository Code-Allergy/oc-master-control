local term = require("term")
local string = require("string")
local component = require("component")
-- local ws = require("./ws_client")
local WebSocket = require("./ws")
local event = require("event")
local internet = require("internet")
local filesystem = require("filesystem")
local thread = require("thread")

local API_KEY_LOCATION = "/home/app/.APIKEY"
local AUTH_KEY_LEN = 8
local API_KEY_LEN = 48
local SERVER_ADDRESS = "http://192.168.1.124:3000"
local WS_ADDRESS = "ws://192.168.1.124"
local WS_PORT = 3000
local WS_PATH = "/api/ws"


-- pull off top of buffer (end), and insert at start
local messageBuffer = {};

-- starting point of execution
function main()
    -- items = component.me_interface.getItemsInNetwork()
--     items = component.me_interface.getItemsInNetwork()
--     if items ~= nil then
--         for i=1, 100 do
--             print(create_json(items[i]))
--         end
--
--
--     end
    if not filesystem.exists(API_KEY_LOCATION) then
        client_initialization()
    end

    local api_keyfile = filesystem.open(API_KEY_LOCATION);
    local api_key = api_keyfile:read(API_KEY_LEN);

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
        if event.pull(1) == 'interrupted' then return end
    end
    print("Connected to WebSocket server!")

    ws:send("Hello, WebSocket!")


    local handle_event = thread.create(event_thread, ws);
    local handle_command = thread.create(command_thread, ws)

    -- Read incoming messages
    while true do
        local messageType, message, err = ws:readMessage()
        if err then return print('Websocket Error: ' .. err) end
        if messageType == WebSocket.MESSAGE_TYPES.TEXT then
            print('Message Received: ' .. message)
            messageBuffer[#messageBuffer + 1] = message
        elseif messageType == WebSocket.MESSAGE_TYPES.PING then
            print('Ping')
            ws:pong(message)
        elseif messageType == WebSocket.MESSAGE_TYPES.PONG then
            print('Pong')
        end

        if event.pull(5) == 'interrupted' then
            ws:close()
            os.exit(1) 
        end
    end

    -- threads for network, ui, other sensors
    -- wait forever
    thread.waitForAll({handle_event, handle_command})
end

function command_thread(ws_client)
    while true do
        if #messageBuffer > 0 then
            -- process message
        end
        os.sleep(1)
    end
end


-- this needs to handle other events and delegate them to the other threads
function event_thread(ws_client)
    while true do
        local ev = {event.pull()};
        print("event: "..ev[1])
        if ev[1] == "interrupted" then
            ws_client:disconnect();
            return;
        elseif ev[1] == "touch" then
            ws_client:send("HELLO!");
        end
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

