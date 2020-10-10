# Building Fingerboard on the Pinephone
On the phone building currently takes a little longer than 40 mins.
You need about 2,6 GB of memory to build Fingerboard. This means you need to activate swap to build it 
if you have the 2 GB edition of the Pinephone. If you have the 3GB version you can skip to step 1.

## Step 0: Activate swap
Make a 1 GB swap file
```bash
sudo dd if=/dev/zero of=/swapfile bs=1M count=1028 status=progress
```
Set the right permissions (a world-readable swap file is a huge local vulnerability):
```bash
sudo chmod 600 /swapfile
```
After creating the correctly sized file, format it to swap:
```bash
sudo mkswap /swapfile
```
Activate the swap file
```bash
sudo swapon /swapfile
```
Finally, edit the fstab configuration to add an entry for the swap file:
```bash
sudo nano /etc/fstab
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
There are some feature flags you might want to set to get more functionalities. Read the [Cargo.toml](../../Cargo.toml) for more information on this. 
Use cargo to build Fingerboard
```bash
cargo build --release
```

If you sent the command via ssh connection, open another one in which you run
```bash
top
```
to prevent the connection from time out. There should not have been any errors, just some warnings, which you can ignore.

## Step 4: Disable Squeekboard
To start fingerboard every time the device boots:
```bash
sudo cp ./packaging/virtboard /usr/bin
```
Squeekboard conflicts with Fingerboard. For testing purposes, you can move the original /usr/bin/squeekboard to a different location and then move it back once you switch back. Phosh crashes if it can not find one of virtboard or squeekboard!

## Step 4 (Optional): Customization
Fingerboards binary comes with a basic layout so you are never left without a possibility to type. You probably want to add a layout for your language or add a stylesheet to customize the look of the keyboard. It's super easy. Here are the [instructions](../../README.md#Customization).