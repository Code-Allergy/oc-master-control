local WebSocket = require("ws");
local filesystem = require("filesystem");
local event = require("event");

local API_KEY_LEN = 48
local API_KEY_LOCATION = "/home/.APIKEY"
local WS_ADDRESS = "ws://192.168.1.124"
local WS_PORT = 3000
local WS_PATH = "/api/ws"

if not filesystem.exists(API_KEY_LOCATION) then
    client_initialization()
end

local api_keyfile = filesystem.open(API_KEY_LOCATION);
local api_key = api_keyfile:read(API_KEY_LEN);

event.listen("interrupted", function()
    ws:close();
    while ws:isOpen() do os.sleep(1) end;
    os.exit(1);
end)

-- setup connection to websocket setver
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

while true do
    local data_read = io.read("*l");
    ws:send("Log " .. tostring(data_read));   
end
