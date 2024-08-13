local event = require("event");


-- run some info functions and send them back to the server
local function cmd_info()
    
end

-- execute update script and download new files from server
local function cmd_update()
    print("Launching updater");
    os.execute("updater");
    print("Throwing an interruption before rebooting for update.");
    event.push("interrupted");
    os.execute("reboot");
end

-- response to a previous command issued. Second token contains the name of original command
local function cmd_response()

end

local Commands = {
    ["Info"]     = cmd_info,
    ["Update"]   = cmd_update,
    ["Response"] = cmd_response,
}




return Commands;


