# crosreleasenotifier
ChromeOS releases commandline.

Install via `cargo install --git`.

## Systemd user service
A systemd user service and timer is provided. To use:
```
cp crosreleasenotifier.{service,timer} ~/.config/systemd/user/
systemctl --user enable --now crosreleasenotifier.timer
```
