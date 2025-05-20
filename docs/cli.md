`myne` also offers a straightforward command-line interface for parsing filenames.

Here's an example:

```console
$ myne "Ascendance of a Bookworm - v07 (2021) (Digital) (Kinoworm).cbz"
```

This command would output the following JSON:

```json
{
  "title": "Ascendance of a Bookworm",
  "digital": true,
  "edited": false,
  "compilation": false,
  "pre": false,
  "revision": 1,
  "volume": "7",
  "chapter": null,
  "group": "Kinoworm",
  "year": 2021,
  "edition": null,
  "extension": "cbz",
  "publisher": null
}
```
