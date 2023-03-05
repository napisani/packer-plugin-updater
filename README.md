cargo build --release
sudo cp target/release/packer_plugin_updater /usr/local/bin/ 

packer_plugin_updater --plugin-lua-file ~/.config/nvim/lua/user/plugins.lua --output-file /tmp/plugins.lua
cp -fv /tmp/plugins.lua ~/.config/nvim/lua/user/plugins.lua 
