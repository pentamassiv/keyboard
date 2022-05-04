[![dependency status](https://deps.rs/repo/github/grelltrier/keyboard/status.svg)](https://deps.rs/repo/github/grelltrier/keyboard)
![Build](https://github.com/grelltrier/keyboard/workflows/Build/badge.svg)
![dependabot status](https://img.shields.io/badge/dependabot-enabled-025e8c?logo=Dependabot)

# WIP: Unfortunetely the keyboard currently does not run on the Pinephone. The language model is too big to share it on Github and it doesn't work without one. I am working on it

# Fingerboard
Fingerboard is an open-source keyboard meant to be used on a smartphone. It serves as a drop-in replacement for squeekboard.
It uses GTK-rs for the GUI and input-method-v2 and virtual-keyboard-v1 to talk to a wayland compositor like phoc/phosh.
It should be fairly easy to add protocols or replace the GUI. Layouts can be easily customized and are loaded when Fingerboard is launched.
Since GTK is used, customizing the looks is also very easy by using CSS and editing the stylesheet. Parts of the code are based on Purism's squeekboard.
I have yet to look up how I properly mention it in the code so they get the props they deserve.

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
- Double press space to enter ". " instead of "  "
- Longpress to capitalize a letter
- Longpress to open popup to select input text (great for our friends of the Umlaut àäâ)
- Automatically show up only when needed and hide when no longer used
- Show when pressing the little keyboard symbol in Phosh
- Detect rotation and switch to different layout
- Make layouts/views partially transparent to make layouts perfect for playing Gameboy emulators

## Installation
If you want to use Fingerboard on your smartphone running Arch, you can [choose a PKGBUILD](packaging/README.md) and install it with pacman. If you use a different distribution, you can easily [build it yourself](docs/building/build_on_pinephone.md) with cargo. 

## Customization
You probably want to use a layout that was created for the language you speak.
To add all available layouts and style, just copy the 'data' folder:

```bash
mkdir ~/.fingerboard
cp -r data ~/.fingerboard
```

If the layout you are looking for is not available, you can easily edit one of the other layouts to adapt it to the missing language. You can't break anything with a malformed layout description. There is a (basic) fallback layout for exactly those cases :). If you made a layout for a missing language, share it with me so others can use it too.

## TODO
So far the code is commented but by the end of october I plan on adding additional diagrams because this is 
a school project. Within the next months I will add next word prediction and gesture typing to it's functionality.

## Debugging
If there are issues with fingerboard, you can set an environment variable and writes to standard error with nice colored output for log levels. There are the debugging levels info, warn and error. Errors are always shown. If you want to for example see all errors but warnings only from the ui_manager module, you would run Fingerboard like this
```bash
RUST_LOG=fingerboard::user_interface::ui_manager=warn ./fingerboard
```
You can also filter the output with regular expressions, turn off colors and more. Read the documentation of [env_logger](https://docs.rs/env_logger) for all options.

## Contributing
Since this is a school project and I don't want to deal with documenting every single line that was not written by myself, I would prefer 
you to open an issue with a description of the mistake I made and possibly a hint on how I can fix it, instead of writing the code for me 
and opening a PRs. That said, this is also my first project using Rust, working with wayland, dbus or most other stuff so if you see how I 
can improve the code quality, I'll be delighted to hear from you. I'll happily accept changes to the stylesheet though :)

## License
[GPL3](https://choosealicense.com/licenses/gpl-3.0/)
