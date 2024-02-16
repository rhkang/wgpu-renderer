# WGPU Renderer

### Run
```bash
cargo run
cargo run --example <EXAMPLE_NAME>
```

### VS code Debugging settings
- install CodeLLDB extension in vscode
- edit launch.json in configuration folder by denoting the target binary
```
// Example
{
    // Use IntelliSense to learn about possible attributes.
    // Hover to view descriptions of existing attributes.
    // For more information, visit: https://go.microsoft.com/fwlink/?linkid=830387
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug executable",            
            "cargo": {
                "args": [
                    "build",
                ],
                "filter": {
                    "name": "wgpu-renderer",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}/target/debug"
        }
    ]
}
```