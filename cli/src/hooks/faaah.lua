-- faaah hook for cmd with Clink: load(io.popen("faaah init cmd"):read("*a"))()

local ran = false

clink.onendedit(function(line)
    ran = line:match("%S") ~= nil
end)

clink.onbeginedit(function()
    local exit_code = os.geterrorlevel()
    if ran and exit_code ~= 0 then
        os.execute("faaah play --detach --exit-code=" .. exit_code)
    end
    ran = false
end)
