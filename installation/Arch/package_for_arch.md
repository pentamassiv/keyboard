# Packaging for Arch
The file osk-wayland needs to be in folder /usr/bin
The file fingerboard needs to be in folder /usr/bin
keyboard-layouts go to folder $HOME/.fingerboard
put .desktop file in /usr/share/applications/

# Starting up keyboard
sm.puri.OSK0.desktop (/usr/share/applications/) is called. This starts /usr/bin/osk-wayland, setting the environment variable to OSK=/usr/bin/virtboard and if squeekboard/fingerboard is available to its bin.
