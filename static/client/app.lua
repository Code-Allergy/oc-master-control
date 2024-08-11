local term = require("term")
local string = require("string")
local component = require("component")
local ws = require("websocket_client")
local event = require("event")

local API_KEY_LOCATION = "/app/.APIKEY"
local AUTH_KEY_LEN = 8
local SERVER_ADDRESS = "http://192.168.1.124:3000"

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
    --client_initialization()

    -- check if apikey file is there, otherwise we should do first time setup
    -- threads for network, ui, other sensors
    -- wait forever
end


function client_initialization()
    while not filesystem.exists("/app/.APIKEY") do
        ::loop::
        term.write("Create an authorization key and enter it here: ")
        result = term.read()
        if #result ~= AUTH_KEY_LEN+1 then
            print("Authorization key shorter than expected")
            goto loop
        end
        auth_code = string.sub(result, 1, -2)
        print(auth_code)
        return "SOME API KEY"
        -- send it


    end

    function get_api_key()

    end
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

