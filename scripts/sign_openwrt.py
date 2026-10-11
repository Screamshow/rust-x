#!/usr/bin/env python3
"""Run on the signing host; the private key never leaves that host."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

p = argparse.ArgumentParser()
p.add_argument('directory', type=Path)
p.add_argument('--apk-tool', required=True)
p.add_argument('--sign-key', required=True)
p.add_argument('--public-key', type=Path, required=True)
a = p.parse_args()
manifest_path = a.directory/'openwrt-manifest.json'
manifest = json.loads(manifest_path.read_text())
with tempfile.TemporaryDirectory(prefix='rust-x-trusted-keys-') as temp:
    shutil.copyfile(a.public_key, Path(temp)/'forkop-apk.pem')
    for asset in manifest['assets']:
        package = a.directory/asset['file']
        original = hashlib.sha256(package.read_bytes()).hexdigest()
        if original != asset['sha256']:
            raise ValueError(f'Input digest mismatch: {package.name}')
        if asset['format'] != 'apk':
            continue
        verified = subprocess.run([a.apk_tool, '--keys-dir', temp, 'verify', str(package.resolve())],
                                  stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        if verified.returncode:
            subprocess.run([a.apk_tool, 'adbsign', '--allow-untrusted', '--sign-key', a.sign_key, str(package.resolve())], check=True)
        subprocess.run([a.apk_tool, '--keys-dir', temp, 'verify', str(package.resolve())], check=True)
        asset.setdefault('unsigned_sha256', original)
        asset['sha256'] = hashlib.sha256(package.read_bytes()).hexdigest()
        asset['size'] = package.stat().st_size
        asset['signed'] = True
manifest['apk_signed'] = True
manifest['signing_public_key_sha256'] = hashlib.sha256(a.public_key.read_bytes()).hexdigest()
shutil.copyfile(a.public_key, a.directory/'forkop-apk.pem')
manifest_path.write_text(json.dumps(manifest, indent=2)+'\n')
(a.directory/'OPENWRT-SHA256SUMS').write_text(''.join(f'{x["sha256"]}  {x["file"]}\n' for x in manifest['assets']))
