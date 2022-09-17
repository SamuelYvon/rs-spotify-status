# rs-spotify-status

> Configurable and easy to use Spotify Status Block command for i3bar 

## Why?

I see a lot of configurations with a simple script like this one to display the current 
song playing on spotify in the i3bar:

```shell
spotify_info=$(dbus-send --print-reply \
    --dest=org.mpris.MediaPlayer2.spotify \
    /org/mpris/MediaPlayer2 \
    org.freedesktop.DBus.Properties.Get \
    string:org.mpris.MediaPlayer2.Player \
    string:Metadata 
)
title=$(echo "$spotify_info" | sed -n '/title/{n;p}' | cut -d '"' -f 2)
artist=$(echo "$spotify_info" | grep artist -A 2 | tail -n 1 | cut -d '"' -f 2)

if [[ "$title" != "" ]]; then
    echo "<span color=\"white\">&#xf1bc; $title ($artist)</span>"
fi
```

This works 99.99% of the time, but not always. It fails on songs with multiple artists
or titles that are absurdly long. Also, it's not in Rust, so it's not blazingly fast (/s).

## Installation + Usage

Install with cargo (`cargo install rs-spotify-status`).

Invoke with `$spotify-status`.

## Configuration

Configuration can be provided through a TOML file at `~/.spotify-status`. An example
configuration follows:

```TOML
icon = # icon string to print. By default, uses the font awesome character code for the spotify icon
color = # the color of the text. By default, white is used.
max_length = # The max length of the string output. By default, 45 UTF-8 characters.
```

