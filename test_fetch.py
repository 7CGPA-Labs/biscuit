import urllib.request
import json
import tarfile
import os

req = urllib.request.Request('https://crates.io/api/v1/crates/webkit6/0.6.1/download', headers={'User-Agent': 'Biscuit/1.0'})
with urllib.request.urlopen(req) as response:
    with open('webkit6.crate', 'wb') as f:
        f.write(response.read())

os.system('tar -xzf webkit6.crate')
os.system('cat webkit6-0.6.1/Cargo.toml | grep links')
