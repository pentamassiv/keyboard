# Fingerboard
Fingerboard is an open-source keyboard meant to be used on a smartphone. It serves as a drop-in replacement for squeekboard.
It uses GTK-rs for the GUI and input-method-v2 and virtual-keyboard-v1 to talk to a wayland compositor like phoc/phosh.
It should be fairly easy to add protocols or replace the GUI. Layouts can be easily customized and are loaded when Fingerboard is lauched.
Since GTK is used, customizing the looks is also very easy by using CSS and editing the stylesheet. Parts of the code are based on Purism's squeekboard.
I have yet to look up how I properly mention it in the code so they get the props they deserve.

## Installation
WARNING: Currently Fingerboard is unusable on versions newer than Arch 20200913. Any help with fixing this is highly appreciated!

If you want to use Fingerboard on your Smartphone running Phosh, you can download the binary or [build it yourself](installation/build_on_pinephone.md) with cargo. If you are building Fingerboard yourself, there are some feature flags you might want to set to get more functionalities.
Read the [Cargo.toml](Cargo.toml) for more information on this. Building it on your phone should be as easy as 
```bash
$ cargo build --release
```
There should not have been any errors, just some warnings, which you can ignore.
Within the next few days I will package it at least for Arch so that it can easily be installed.
You can try Fingerboard by replace the squeekboard binary (/usr/bin/squeekboard) with a dummy one so squeekboard is not started. This will prevent squeekboard from comflicting with Fingerboard. A simple hello world binary is enough but it needs to be executable. Then you can launch Fingerboard via SSH and if you are done testing it you can put the squeekboard binary back.

When Fingerboard is launched, it looks for subfolders for a stylesheet, user defined layouts and custom icons in the folder
```bash
~/.fingerboard
```
It's easiest to copy the folders 'data' and 'theming' from the repository and paste it in that folder. You can also skip this step to get the fallback layout.

## Features
- Visual/Haptic feedback
- Customizable layouts/skin
- Input text (Unicode)
- Input keycodes
- Input modifiers
- Input emoji 😍
- Input smileys ᕙ( ͡° ͜ʖ ͡°)ᕗ
- Toggle keys
- Switch layouts/views/layer
- Double press space to enter ". " instead of " "
- Longpress to capitalize a letter
- Longpress to open popup to select input text (great for our friends of the Umlaut àäâ)
- Automatically show up only when needed and hide when no longer used
- Show when pressing the little keyboard symbol in Phosh
- Detect rotation and switch to different layout
- Make layouts/views partially transparent to make layouts perfect for playing Gameboy emulators

## TODO
So far the code is not really commented but by the end of October I plan on adding tons of diagrams and comments because this is 
a school project. Within the next months I will add next word prediction and gesture typing to it's functionality.

## Contributing
Since this is a school project and I don't want to deal with documenting every single line that was not written by myself, I would prefer 
you to open an issue with a description of the mistake I made and possibly a hint on how I can fix it, instead of writing the code for me 
and opening a PRs. That said, this is also my first project using Rust, working with wayland, dbus or most other stuff so if you see how I 
can improve the code quality, I'll be delighted to hear from you. I'll happily accept changes to the stylesheet though :)

## License
[GPL3](https://choosealicense.com/licenses/gpl-3.0/)

