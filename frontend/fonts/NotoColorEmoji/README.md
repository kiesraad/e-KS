`NotoColorEmojiFlags.woff2` is a subset of the [Noto Color Emoji](https://fonts.google.com/noto/specimen/Noto+Color+Emoji) font that only contains the emoji flag symbols.

The subset was created using [pyftsubset](https://fonttools.readthedocs.io/en/stable/subset/):

```sh
pyftsubset ./NotoColorEmoji-Regular.ttf --unicodes=1F1E6-1F1FF --output-file=Flags.woff2 --flavor=woff2
```
