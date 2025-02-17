SERVER = "localhost"
USERNAME = "ErrorNoWatcher"
OWNERS = { "ErrorNoInternet" }

for _, module in
	{
		"enum",
		"events",
		"utils",
	}
do
	module = "lib/" .. module
	package.loaded[module] = nil
	require(module)
end
