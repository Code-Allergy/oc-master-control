-- this will be added into app later, for now it is static update app.
-- fir now, output_dir is only one file.
local internet = require("internet")
local filesystem = require("filesystem")

local OUTPUT_DIR = "app"


local UPDATE_URL = "http://192.168.1.124:3000/static"
local CLIENT_UPDATE_URL = UPDATE_URL .. "/client/latest"
local UPDATER_UPDATE_URL = UPDATE_URL .. "/updater/latest"


-- this will get updated in the script if there is a newer version
local version_no = 0.00

print("Controller Updater v0.02")
-- print("Checking if", OUTPUT_DIR, "directory already exists")
-- if filesystem.exists(OUTPUT_DIR) then
    -- if not filesystem.exists(OUTPUT_DIR .. "/VERSION") then
    --     error("/app folder exists but there is no version file. 
    --     \nEither remove the folder or fix the version file.")
    -- end
--     version_file = filesystem.open(OUTPUT_DIR .. "/VERSION", "r")
--     version_no = version_file.read(8)
--     version_file.close()
-- end

-- request update

function updateFile(url, filename)
    local req_handle = internet.request(url)
    local result = ""
    for chunk in req_handle do result = result..chunk end

    local new_file = filesystem.open(filename, "w")
    if new_file == nil then
        error("Error creating output file " .. filename)
    end

    if new_file:write(result) then
        print("Success!")
    else
        error("Failed to update file"..filename.."!")
    end
end

updateFile(CLIENT_UPDATE_URL, "/home/app")
updateFile(UPDATER_UPDATE_URL, "/home/updater")
