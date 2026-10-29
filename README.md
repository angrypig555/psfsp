# :floppy_disk: Pretty Simple File Sharing Protocol
> [!CAUTION]
> this project is not meant to be used in production environments, where security is important

## how does it work
psfsp is a protocol built from scratch using encrypted TCP sockets.
The client greets the server and the server redirects the client to a new, ephemeral port.
After the client is transferred, it can query files and initiate a download or upload.
Clients who don't authenticate are automatically ignored

## how 2 run
> [!NOTE]
> the gui sometimes throws errors even when a download succeded, always double check file integrity

you can follow any of these instructions to install psfsp

### Option A, Compile from scratch
first, build the executable

`cargo build --release`

and navigate to the target/release folder for the executable
to build the gui, go into `psfsp_gui` and install tauri-cli

`cargo install tauri-cli --locked`

then you can build the gui

`cargo tauri build`

### Option B, Download the prebuilt executable
you can download a prebuilt executable for linux, windows and macos from the releases page
> [!IMPORTANT]
> only tested on linux, may not work on windows and macos
> 
### Option C, Download from crates.io
both the client and server are uploaded to crates.io and can be installed by running

`cargo install psfsp`

if you require the gui, it must be built manually from source as it relies on system webview libraries, which is different on every platform

### Arguments
Client:
`client [server_ip] [download_directory]`

Server:
`server [shared_directory] [username] [password]`

all arguments are required and must be in the same order as described here

## :robot: AI Notice
some AI was used to help with debugging the code.
***THIS README WAS <ins>NOT</ins> WRITTEN BY AI.***
