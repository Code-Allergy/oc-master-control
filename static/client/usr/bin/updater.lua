local internet = require("internet");
local filesystem = require("filesystem");
local util = require("util");

local CLIENT_UPDATE_URL = "http://192.168.1.124:3000/static/client"
local UPDATE_MANIFEST_URL = CLIENT_UPDATE_URL .. "/MANIFEST"

local function get_update_urls()
    local req_handle = internet.request(UPDATE_MANIFEST_URL)
    local result = ""
    for chunk in req_handle do result = result..chunk end
    return util.split_on_newl(result);
end

local function updateFile(url, filename)
    local req_handle = internet.request(url);
    local result = "";
    for chunk in req_handle do result = result..chunk end;

    local new_file = filesystem.open(filename, "w");
    if new_file == nil then
        error("Error creating output file " .. filename);
    end

    if not new_file:write(result) then
        error("Failed to update file"..filename.."!")
    end
end

local function main()
    local updates = get_update_urls()
    for idx, file in ipairs(updates) do
        updateFile(CLIENT_UPDATE_URL.."/"..file, file)
    end
end

main()
