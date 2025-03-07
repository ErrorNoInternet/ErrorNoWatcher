Server = "localhost"
Username = "ErrorNoWatcher"
Owners = { "ErrorNoInternet" }

for _, module in ipairs({
	"automation",
	"enum",
	"events",
	"inventory",
	"lib",
	"movement",
	"utils",
}) do
	module = "lib/" .. module
	package.loaded[module] = nil
	require(module)
end

update_listeners()
