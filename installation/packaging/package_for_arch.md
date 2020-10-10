# Packaging for Arch
The executable file fingerboard needs to be in folder /usr/bin
The executable file virtboard needs to be in folder /usr/bin
keyboard-layouts go to folder $HOME/.fingerboard

## Starting up a virtual keyboard under phosh
sm.puri.OSK0.desktop (/usr/share/applications/) is called by phosh. This starts /usr/bin/osk-wayland, setting the environment variable to OSK=/usr/bin/virtboard and if squeekboard/fingerboard is available to /usr/bin/squeekboard. Then it executes $OSK
