#!/usr/bin/env python3

import urllib.request
import hashlib
import os

def main():
    name = "lychee-v2"
    hash = "920e695ad495ba8fb8a9e28afad1644e03d26ae471e29267aea630615be22b2d"
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
