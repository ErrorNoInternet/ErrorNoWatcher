Server = "localhost"
Username = "ErrorNoWatcher"
Owners = { "ErrorNoInternet" }

for _, module in ipairs({
	"enum",
	"events",
	"inventory",
	"movement",
	"utils",
}) do
	module = "lib/" .. module
	package.loaded[module] = nil
	require(module)
end
