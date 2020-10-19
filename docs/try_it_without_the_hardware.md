# How to try it without having the hardware

The application needs a compositor that understands the input_method_unstable_v2 and the virtual_keyboard_unstable_v1 wayland protocols. If your compositor does not meet the requirements,
you can still try it with a virtual machine.

Get the newest build file from [Purisms](https://arm01.puri.sm/job/Images/job/Image%20Build/) to emulate the OS (Tested with Build [#5656](https://arm01.puri.sm/job/Images/job/Image%20Build/5656/)).

Download the file qemu-x86_64.qcow2. It is about 2,75 GB to download.

Use your favorite program that can create a virtual machine from a qcow2 image. GNOME Boxes is one option.
Select Debian Buster as the OS. There currently is an issue with the mouse. You need to click a little more to the right, than where you intend to click.

The password is 123456.

Execute the install.sh script (This will reboot the VM)
```bash
wget https://github.com/grelltrier/keyboard/releases/download/v0.16-x86/install.sh
chmod +x install.sh
./install.sh
```
