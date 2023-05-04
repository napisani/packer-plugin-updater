## What is packer_plugin_updater

`packer_plugin_updater` is a CLI tool used for updating pinned packer/neovim plugins defined in a plugin.lua file. 
`packer_plugin_updater` uses a lua AST to parse the plugin.lua module into a list of defined plugins. Then, the tool will compare the pinned commit hash to 
the latest commit hash in the remote git repository. 

## download pre-built binary
Pre-built binaries are available here for download: [HERE](https://github.com/napisani/packer-plugin-updater/releases)
## build
Use this command to build the binary from source: 
```bash
cargo build --release
```





