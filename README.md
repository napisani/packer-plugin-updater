## What is packer_plugin_updater

`packer_plugin_updater` is a CLI tool used for updating pinned packer/neovim plugins defined in a plugin.lua file. 
`packer_plugin_updater` uses a lua AST to parse the plugin.lua module into a list of defined plugins. Then, the tool will compare the pinned commit hash to 
the latest commit hash in the remote git repository. 

## Download pre-built binary
Pre-built binaries are available here for download: [HERE](https://github.com/napisani/packer-plugin-updater/releases)

## build
Use this command to build the binary from source: 

```bash
cargo build --release
```

## usage

First off, it is important to note `packer_plugin_updater` requires a `define_plugins` function at the top-level of the plugin.lua module.
This function should have a series of `use` calls using the this syntax (this no different from the syntax recommended in the packer docs).
```lua
...
local function define_plugins(use)
  -- Have packer manage itself
  use { "wbthomason/packer.nvim", commit = "1d0cf98a561f7fd654c970c49f917d74fafe1530" }

  -- Useful lua functions used by lots of plugins
  use { "nvim-lua/plenary.nvim", commit = "253d34830709d690f013daf2853a9d21ad7accab" }

  -- Autopairs, integrates with both cmp and treesitter
  use { "windwp/nvim-autopairs", commit = "e755f366721bc9e189ddecd39554559045ac0a18" }

...
end

```

The 'packer_plugin_updater' cli can be used liked this to interactively upgrade each outdated dependency individually:
```bash
./target./release/packer_plugin_updater --plugin-lua-file ~/.config/nvim/lua/user/plugins.lua --output-file /tmp/plugins.lua

```
Alternatively,the cli can be used liked this to upgrade all plugins to the latest.
```bash
./target./release/packer_plugin_updater --plugin-lua-file ~/.config/nvim/lua/user/plugins.lua --output-file /tmp/plugins.lua --update-all
```

Next, you can review the output file and if you are happy with it, you can copy it into place and replace the original plugin.lua file
```bash
cp /tmp/plugins.lua ~/.config/nvim/lua/user/plugins.lua
# start nvim
nvim

# within nvim, run this to sync the new versions using packer
:PackerSync
```
