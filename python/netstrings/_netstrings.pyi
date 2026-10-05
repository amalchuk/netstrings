from collections.abc import Buffer

__all__: tuple[str, ...] = (
    "dumps",
    "loads",
)

def dumps(data: Buffer, /) -> bytes: ...
def loads(data: Buffer, /) -> bytes: ...
