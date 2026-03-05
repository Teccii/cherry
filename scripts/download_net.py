#!/usr/bin/env python3

import urllib.request
import hashlib
import os

def main():
    name = "lychee-v4"
    hash = "3b27906983ec6b97f848a82259d359e69b7c5eaaf2144e38a3e2349502ec3f8f"
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
