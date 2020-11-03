# Packaging for Arch
There are two packaging options for you to get Fingerboard. You can use a package that [build it from source](src) or you can just install the [compiled binaries](bin). If you don't know what this means, you want the compiled binaries.
To get the PKGBUILD, run
```bash
wget https://raw.githubusercontent.com/grelltrier/keyboard/master/packaging/bin/PKGBUILD
```
or
```bash
wget https://raw.githubusercontent.com/grelltrier/keyboard/master/packaging/src/PKGBUILD
```

Once you have the PKGBUILD, run
```bash
makepkg
pacman -U [NAME_OF_PACKAGE]
```
to install it on your device.