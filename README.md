# myne

[![Tests](https://img.shields.io/github/actions/workflow/status/Ravencentric/myne/tests.yml?label=tests)](https://github.com/Ravencentric/myne/actions/workflows/tests.yml)
[![Build](https://img.shields.io/github/actions/workflow/status/Ravencentric/myne/release.yml?label=build)](https://github.com/Ravencentric/myne/actions/workflows/release.yml)
![PyPI - Types](https://img.shields.io/pypi/types/myne)
![License](https://img.shields.io/pypi/l/myne?color=success)

[![PyPI - Latest Version](https://img.shields.io/pypi/v/myne?color=blue)](https://pypi.org/project/myne)
![PyPI - Python Version](https://img.shields.io/pypi/pyversions/myne)
![PyPI - Implementation](https://img.shields.io/pypi/implementation/myne)

`myne` is a parser for manga and light novel filenames, providing the `Book` class for extracting and representing metadata such as title, volume, chapter, edition, and more.

## Usage

```python
from myne import Book

book = Book.parse("Ascendance of a Bookworm - v07 (2021) (Digital) (Kinoworm).cbz")

assert book.title == "Ascendance of a Bookworm"
assert book.volume == "7"
assert book.year == 2021
assert book.digital is True
assert book.group == "Kinoworm"
```

Checkout the complete documentation [here](https://ravencentric.cc/myne/).

## Installation

`myne` is available on [PyPI](https://pypi.org/project/myne/), so you can simply use pip to install it.

```console
pip install myne
```

## Building from source

Building from source requires the [Rust toolchain](https://rustup.rs/) and [Python 3.9+](https://www.python.org/downloads/).

- With [`uv`](https://docs.astral.sh/uv/):

  ```console
  git clone https://github.com/Ravencentric/myne
  cd myne
  uv build
  ```

- With [`pypa/build`](https://github.com/pypa/build):

  ```console
  git clone https://github.com/Ravencentric/myne
  cd myne
  python -m build
  ```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/Ravencentric/myne/blob/main/LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://github.com/Ravencentric/myne/blob/main/LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

## Contributing

Contributions are welcome! Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
