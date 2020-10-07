# Building Fingerboard on the Pinephone

You need about 2,2 GB of memory to build Fingerboard. This means you need to activate swap to build it 
if you have the 2 GB edition of the Pinephone. If you have the 3GB version you can skip to step 1.

## Step 0: Activate swap
Make a 1 GB swap file
```bash
dd if=/dev/zero of=/swapfile bs=1M count=1028 status=progress
```
Set the right permissions (a world-readable swap file is a huge local vulnerability):
```bash
chmod 600 /swapfile
```
After creating the correctly sized file, format it to swap:
```bash
mkswap /swapfile
```
Activate the swap file
```bash
swapon /swapfile
```
Finally, edit the fstab configuration to add an entry for the swap file:
```bash
/etc/fstab
/swapfile none swap defaults 0 0
```

## Step 1: Get needed packages
Install necessary packages for building:
```bash
sudo pacman -S gcc pkgconf gtk-layer-shell
```

## Step 2: Get Rust
Fingerboard is written in Rust so the easiest way to build is it to use Cargo. To get everything you need, it is recommended to use rustup:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Add Cargo's bin directory ($HOME/.cargo/bin) to your PATH environment variable
```bash
source $HOME/.cargo/env
```

## Step 3: Build Fingerboard
Clone Fingerboards repository
```bash
git clone https://github.com/grelltrier/keyboard.git
cd ./keyboard
```

Use Cargo to build Fingerboard
```bash
cargo build --release
```

If you sent the command via ssh connection, open another one in which you run
```bash
top
```
to prevent the connection from time out