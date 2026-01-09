# Langtu Controller (L98)
- Quickly cobbled solution made to control the RGB on my keyboard.
- ***THIS MAY NOT WORK ON ANY COMPUTER BUT MINE!!!*** 

## Install/Build
- Requires cargo on path, with a relevant toolchain installed.
- This program needs root to write directly to hidraw, so you can either:
  - **Create a udev rule (recommended)**: [read here](#udev-configuration)
  - Always run it with sudo
- Run `chmod +x ./build.fish && ./build.fish`
  - Will ask for sudo permission to copy to `/usr/bin/langtuctl`

## Usage
- Command takes form: `langtuctl <MODE> <VALS...> <BRIGHTNESS>`
- Mode is either `rainbow`,`color` or `complete`
- **Rainbow:**
  - `DIRECTION` is direction of rainbow, odd is to the left, even is to the right [0-255]
  - `SPEED` is speed of rainbow cycling [0-4]
  - `BRIGHTNESS`...controls brightness [0-4]
- **Color:**
  - `RED` is the R component of color. [0-255]
  - `GREEN` is the G component of color. [0-255]
  - `BLUE` is the B component of color. [0-255]
  - `BRIGHTNESS`...controls brightness [0-4]

## Udev Configuration
1. Create and edit rule file via `sudo nano /etc/udev/rules.d/99-langtu.rules`
2. Paste in (makes it so only users of group "langtumod" can modify the hidraws):
> \# Langtu L98 keyboard  
> SUBSYSTEM=="hidraw", ATTRS{idVendor}=="1a2c", ATTRS{idProduct}=="7fff", MODE="0660", GROUP="langtumod"``
3. Add profile to group:`sudo usermod -aG langtumod $USER`
4. Restart system
5. Check `groups` and ensure `langtumod` is listed

## Autocomplete
1. Very basic autocomplete can be added by sourcing `langtuctl completion $SHELL`
