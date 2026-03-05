#!/usr/bin/env python3

import urllib.request
import hashlib
import os

def main():
    name = "lychee-v3"
    hash = "1676c193fc9e686bad9f9bea0bb93ef8875b3c1e04ca94f78155104789848b36"
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
