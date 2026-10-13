"""Escape Rust fragments and format complete generated files."""

import subprocess

def literal(value: str) -> str:
    """Quote a Rust string literal, including control characters and Unicode."""
    escapes = {"\\": "\\\\", '"': '\\"', "\n": "\\n", "\r": "\\r", "\t": "\\t", "\0": "\\0"}
    result = []
    for char in value:
        if 0xD800 <= ord(char) <= 0xDFFF:
            raise ValueError("Rust strings require Unicode scalar values")
        result.append(escapes.get(char, f"\\u{{{ord(char):x}}}" if ord(char) < 32 or ord(char) == 127 else char))
    return '"' + "".join(result) + '"'


def markdown(value: str) -> str:
    return value.replace("\\", "\\\\").replace("[", "\\[").replace("]", "\\]")


def format_rust(source: str) -> str:
    result = subprocess.run(
        ["rustfmt", "--edition", "2024", "--emit", "stdout"],
        input=source,
        text=True,
        capture_output=True,
        check=True,
    )
    return result.stdout
