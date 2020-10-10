# Packaging for Arch
There are two packaging options for you to get Fingerboard. You can build it yourself [build it yourself](/src) or you can just install the [compiled binaries](/bin). If you don't know what this means, you want the compiled binaries.

## Starting up a virtual keyboard under phosh
sm.puri.OSK0.desktop (/usr/share/applications/) is called by phosh. This starts /usr/bin/osk-wayland, setting the environment variable to OSK=/usr/bin/virtboard and if squeekboard/fingerboard is available to /usr/bin/squeekboard. Then it executes $OSK. This is why Fingerboard puts a moves a script to /usr/bin/virtboard that calls Fingerboard.