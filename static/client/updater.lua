-- this will be added into app later, for now it is static update app.
-- fir now, output_dir is only one file.
local internet = require("internet")
local filesystem = require("filesystem")

local OUTPUT_DIR = "/home/app"


local UPDATE_URL = "http://192.168.1.124:3000/static"
local CLIENT_UPDATE_URL = UPDATE_URL .. "/client"
local UPDATER_UPDATE_URL = UPDATE_URL .. "/updater/latest"

local UPDATE_MANIFEST_URL = UPDATE_URL .. "/client/MANIFEST"


-- this will get updated in the script if there is a newer version
local version_no = 0.00

print("Controller Updater v0.04")

function get_update_urls()
    local req_handle = internet.request(UPDATE_MANIFEST_URL)
    local result = ""
    for chunk in req_handle do result = result..chunk end
    local lines = splitByNewline(result)

    local files = {}
    for i, line in ipairs(lines) do
        file, version = splitBySpace(line)
        files[file] = version
    end

    return files
end

function updateFile(url, filename)
    local req_handle = internet.request(url)
    if req_handle == nil then
        error("Failed to connect to "..url)
    end
    local result = ""
    for chunk in req_handle do result = result..chunk end

    local new_file = filesystem.open(filename, "w")
    if new_file == nil then
        error("Error creating output file " .. filename)
    end

    if new_file:write(result) then
        print("Success!"..filename.." was updated successfully!")
    else
        error("Failed to update file"..filename.."!")
    end
end

function splitByNewline(str)
    local result = {}
    for line in string.gmatch(str, "[^\r\n]+") do
        table.insert(result, line)
    end
    return result
end

function splitBySpace(str)
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

local updates = get_update_urls()
for file, version in pairs(updates) do
    updateFile(CLIENT_UPDATE_URL.."/"..file, OUTPUT_DIR.."/"..file)
end