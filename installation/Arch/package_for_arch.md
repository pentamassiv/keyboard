# Packaging for Arch
The file fingerboard needs to be in folder /usr/bin
The file virtboard needs to be in folder /usr/bin
keyboard-layouts go to folder $HOME/.fingerboard
put .desktop file in /usr/share/applications/

## Starting up virtual keyboard under phosh
sm.puri.OSK0.desktop (/usr/share/applications/) is called by phosh. This starts /usr/bin/osk-wayland, setting the environment variable to OSK=/usr/bin/virtboard and if squeekboard/fingerboard is available to /usr/bin/squeekboard. Then it executes $OSK
