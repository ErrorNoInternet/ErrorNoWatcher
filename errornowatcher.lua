Server = "localhost"
Username = "ErrorNoWatcher"
HttpAddress = "127.0.0.1:8080"
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
