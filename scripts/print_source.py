# List the filename and content of files in the ./src directory
# Usage: python print_source.py

import os


def print_source():
    for root, dirs, files in os.walk("./src"):
        for file in files:
            with open(os.path.join(root, file), "r") as f:
                print(f"{file}")
                print(f"```rust\n{f.read()}\n```")
                print()


__name__ == "__main__" and print_source()
