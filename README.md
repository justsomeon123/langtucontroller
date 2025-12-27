# Langtu Controller (L98)

- Quickly cobbled solution made to control the RGB on my keyboard.
- ***THIS MAY NOT WORK ON ANY COMPUTER BUT MINE!!!*** 

## Install
- Requires cargo on path, with a relevant toolchain installed.
- This program needs root to write directly to hidraw, so you can either:
  - Always run it with sudo
  - Create a udev rule: [read here](#udev-configuration)
- Run `chmod +x ./build.fish && ./build.fish`
  - Will ask for sudo permission to copy to `/usr/bin/langtuctl`

## Usage
- Command takes form: `langtuctl <MODE> <VAL1> <VAL2> <VAl3> <BRIGHTNESS>`
- Mode is either `rainbow` or `color`
- **Rainbow:**
  - `VAL1` is direction of rainbow, odd is to the left, even is to the right [0-255]
  - `VAL2` is speed of rainbow [0-4]
  - `VAl3` is irrelevant [0-255]
  - `BRIGHTNESS`...controls brightness [0-4]
- **Color:**
  - `VAL1` is the R component of color. [0-255]
  - `VAL2` is the G component of color. [0-255]
  - `VAl3` is the B component of color. [0-255]
  - `BRIGHTNESS`...controls brightness [0-4]

## Udev Configuration
1. Create and edit rule file via `sudo nano /etc/udev/rules.d/99-langtu.rules`
2. Paste in (makes it so only users of group "langtumod" can modify the hidraws):
> \# Langtu L98 keyboard  
> SUBSYSTEM=="hidraw", ATTRS{idVendor}=="1a2c", ATTRS{idProduct}=="7fff", MODE="0660", GROUP="langtumod"``
3. Add profile to group:`sudo usermod -aG langtumod $USER`
4. Restart system
5. Check `groups` and ensure `langtumod` is listed
