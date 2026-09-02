# :floppy_disk: Pretty Simple File Sharing Protocol
> [!CAUTION]
> This application is not meant to be used in production environments, as security is not guaranteed

## :question: How does it work?
psfsp is a protocol built from scratch using encrypted TCP sockets.
The client greets the server and the server redirects the client to a new, ephemeral port.
After the client is transferred, it can query files and initiate a download or upload.
Clients who don't authenticate are automatically ignored

## How do i run this?
> [!NOTE]
> Due to how the GUI works, sometimes it throws errors even when it succeded, always double check if your download succeded!

psfsp requires no external dependencies.
You can follow either options to run psfsp

### :computer: Option A, Compile from scratch
To build, run
`cargo build --release`
and navigate to the target/release folder for the executable
To build the GUI application, go into `psfsp_gui` and install tauri-cli
`cargo install tauri-cli --locked`
And then you can build the application
`cargo tauri build`
### :package: Option B, Download the prebuilt executable
You can download a prebuilt executable for linux, windows and macOS from the releases page
> [!IMPORTANT]
> This has only been tested on windows and linux, technically it should work on macOS but it is not guaranteed

### 💾 Option C, Download from crates.io
The client and the server binaries can be downloaded from crates.io by running

`cargo install psfsp`

The GUI has to be built manually because it requires the system webview libraries.

### Arguments
Client:
`client [server_ip] [download_directory]`

Server:
`server [shared_directory] [username] [password]`

All arguments are required and they must be in the correct order

## :robot: AI Notice
Some AI was used to help with debugging the code.
This readme was not written by AI.
