# `netstrings-py`

A high-performance [`netstrings`][1] encoding and decoding library for Python.

## Requirements

* Python **3.12** and above.
* Rust **1.91** and above.

## Example Usage

```pycon
>>> import netstrings
>>>
>>> netstrings.dumps(b"Hello, World!")
b'13:Hello, World!,'
>>>
>>> netstrings.loads(b"13:Hello, World!,")
b'Hello, World!'
```

## Distribution

This project is licensed under the terms of the [MIT License](LICENSE).

[1]: <https://cr.yp.to/proto/netstrings.txt>
