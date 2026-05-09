#!/usr/bin/env python3

import urllib.request
import hashlib
import os

def main():
    name = "salak-v2"
    hash = "5fb54fb22b55ed7a3e3bf26cc139ef069d0bb80bca14e36cfc00d3ca3618bb30"
    path = "./networks/default.nnue"

    os.makedirs(os.path.dirname(path), exist_ok=True)

    try:
        if hashlib.sha256(open(path, "rb").read()).digest().hex() == hash:
            print("Net already exists!")
            return
    except OSError:
        pass

    print(f"Downloading net {name} to {path}")
    net = urllib.request.urlopen(
        f"https://github.com/Teccii/cherry-networks/releases/download/{name}/{name}.nnue"
    ).read()
    if hashlib.sha256(net).digest().hex() != hash:
        print("Invalid hash!")
        exit(1)
    open(path, "wb").write(net)

if __name__ == "__main__":
    main()
