# Starting up a virtual keyboard under phosh (in Arch)

sm.puri.OSK0.desktop (/usr/share/applications/) is called by phosh. This starts /usr/bin/osk-wayland, setting the environment 
variable to OSK=/usr/bin/virtboard and if squeekboard/fingerboard is available to /usr/bin/squeekboard. Installing fingerboard overwrites /usr/bin/osk-wayland. This new script will additionally check if /usr/bin/osk-overwrite exists. If it does, it sets the OSK variable to that path. Then it executes $OSK. 
This is why Fingerboard moves its executable to /usr/bin/fingerboard and adds a symlink from /usr/bin/osk-overwrite to it.