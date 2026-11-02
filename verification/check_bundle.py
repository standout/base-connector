#!/usr/bin/env python3
"""Offline integrity check only: never loads WASM, never authenticates provenance."""
import argparse
import hashlib
import json
from pathlib import Path


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check(bundle, expected_artifact, expected_commit, allow_dirty=False):
    def require(condition, message):
        if not condition:
            raise ValueError(message)

    manifest = json.loads((bundle / 'manifest.json').read_text(encoding='utf-8'))
    require(manifest['format'] == 'integrationer.verified-wasm-fixture.v1', 'Unknown format')
    require(manifest['test_only'] is True and manifest['publishable'] is False, 'Not an internal fixture')
    require(manifest['source_commit'] == expected_commit, 'Unexpected source commit')
    require(allow_dirty or manifest['source_dirty'] is False, 'Dirty source evidence')
    require(manifest['artifact']['file'] == 'connector.wasm', 'Unexpected artifact path')
    require(manifest['artifact']['sha256'] == expected_artifact == digest(bundle / 'connector.wasm'), 'Artifact digest mismatch')
    require(manifest['artifact']['size'] == (bundle / 'connector.wasm').stat().st_size, 'Artifact size mismatch')
    require(manifest['contract']['file'] == 'contract.json' and manifest['tests']['file'] == 'tests.json', 'Unexpected evidence path')
    contract = json.loads((bundle / 'contract.json').read_text(encoding='utf-8'))
    canonical = json.dumps(contract, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
    require(hashlib.sha256(canonical).hexdigest() == manifest['contract']['sha256'], 'Contract digest mismatch')
    require(digest(bundle / 'tests.json') == manifest['tests']['sha256'], 'Tests digest mismatch')
    tests = json.loads((bundle / 'tests.json').read_text(encoding='utf-8'))
    require(tests['status'] == manifest['tests']['status'] == 'passed', 'Tests did not pass')
    require(tests['source_commit'] == expected_commit and tests['source_dirty'] == manifest['source_dirty'], 'Tests source mismatch')
    require(tests['artifact_sha256'] == expected_artifact and tests['contract_sha256'] == manifest['contract']['sha256'], 'Tests identity mismatch')
    require({case['scenario'] for case in tests['cases']} == {'contract', 'network', 'cpu', 'memory', 'wrong_digest_rejected_before_load'}, 'Missing required cases')
    require(all(case['status'] == 'passed' for case in tests['cases']), 'Failed case')
    return manifest


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('bundle', type=Path)
    parser.add_argument('--expected-artifact', required=True, help='Expected SHA-256 from an independently selected build')
    parser.add_argument('--expected-commit', required=True)
    args = parser.parse_args()
    check(args.bundle, args.expected_artifact, args.expected_commit)
    print('Integrity verified; unsigned evidence does not establish trusted provenance or publication permission.')
