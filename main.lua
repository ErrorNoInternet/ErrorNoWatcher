Server = "localhost"
Username = "ErrorNoWatcher"
HttpAddress = "127.0.0.1:8080"
Owners = { "ErrorNoInternet" }
MatrixOptions = { owners = { "@errornointernet:envs.net" } }

for _, module in ipairs({
	"lib",
	"automation",
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

update_listeners()
