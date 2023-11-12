<div align="center">
  <img src=".github/resources/banner.gif" width=75%>
  <div>
    <em>🌊 a thug pro trickset copier.</em>
  </div>
</div>

## 📸 Screenshots

![](.github/resources/screenshots.png)

## ⏬ Installation

> [!WARNING]  
> Please make a manual backup of your saves! I've tested this along with a few community
> members and have yet to observe any issues, but your results may vary, as cascade is
> still experimental software.

Download `cascade.exe` from the
[latest release on GitHub](https://github.com/1borgy/cascade/releases/latest).

Cascade will automatically detect your THUG Pro installation directory if it is in
`%localappdata%/THUG Pro/`. If you have it installed elsewhere, you will need to
[configure cascade](#%EF%B8%8F-configuration).

## 🤠 Usage

Using cascade is very easy:

- Click "set trickset" to tell cascade which CAS's trickset you want to use.
- Click "copy trickset to saves" to copy that trickset to all of your save files.

### Restoring Backups

If you want to revert copying a trickset, or the saves were corrupted for some reason,
cascade stores backups at `THUG Pro/.cascade/` every time it copies tricksets. Each
entry in this directory is marked with a date and time so you know when it was backed
up. Simply pick which backup you want to use and drag its contents into your
`THUG Pro/Save/` directory.

## ⚙️ Configuration

If your THUG Pro is installed elsewhere, you will need to tell cascade where to look
for saves. Simply click on the "config" tab and select the path for "thugpro saves".
This path should point to the `THUG Pro/Save/` directory, not the base `THUG Pro/`
directory.

### Advanced Configuration

By default, cascade stores all its required files in `THUG Pro/.cascade/`. Currently the
cascade config (`cascade.toml`) cannot be moved from this directory, but that is likely
to change if there is demand for it (e.g. a portable install where it's stored in the
same directory as the executable).

If you want to copy the trickset to just some of your saves, you can move them to a
different directory and set the "thugpro saves" path to that directory instead.

## 📢 Feedback

If something isn't working for you, please let me know! My goal is for everyone to use
those old CASes they made but don't want to use because they have an outdated trickset.

Please reach out to me on discord @triplecheeseborger if you have any issues getting
cascade to work. If you run into an issue, please attach the cascade logs (you can find
these at `THUG Pro/.cascade/cascade.log`) so I can help diagnose the issue.

I'd also love to hear feedback! Things like:

- What else do you want to see in cascade?
- How can the user interface be improved?
- Is the UI's performance good on your system?
- etc.

## ✨ Shoutouts

Cascade would not exist without the following people, so shoutout to them:

- source for the great work on [castool](https://castool.xyz) and for giving me pointers
  regarding CAS format
- [@chc](https://github.com/chc) for the great work on the
  [save editor](http://save-editor.thmods.com/#/manage_save)
  and [THPS.API](https://github.com/chc/thps.api/tree/master)
- c4m 🦆 for being the first beta tester, making the project logo and banner, and
  helping me stay sane during development ✨
- retro 🦆 for being an early beta tester
- [@wermipls](https://github.com/wermipls) for helping with github actions
