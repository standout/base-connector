#!/usr/bin/env python3
"""Offline integrity check only: never loads WASM, never authenticates provenance."""
import argparse
import hashlib
import json
from pathlib import Path


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check(bundle, expected_artifact, expected_commit, allow_dirty=False, candidate=False):
    def require(condition, message):
        if not condition:
            raise ValueError(message)

    manifest = json.loads((bundle / 'manifest.json').read_text(encoding='utf-8'))
    if candidate:
        require(manifest['format'] == 'integrationer.verified-wasm-candidate.v1', 'Not a candidate bundle')
        require(manifest['artifact_role'] == 'internal_connector_candidate', 'Wrong artifact role')
        require(all(manifest[key] is False for key in ('published', 'origin_verified', 'publication_authorized')), 'Unsupported authorization claim')
    else:
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
    required_cases = {'candidate', 'wrong_digest_rejected_before_load'} if candidate else {'contract', 'network', 'cpu', 'memory', 'wrong_digest_rejected_before_load'}
    require({case['scenario'] for case in tests['cases']} == required_cases, 'Missing required cases')
    if candidate:
        require(contract['action'] == 'echo_contract' and contract['diagnostic_actions'] == [], 'Unexpected candidate actions')
        environment = {}
        for name in ('manifest', 'tests'):
            entry = manifest['environment_probes'][name]
            require(entry['file'] == f'environment-{name}.json', 'Unexpected environment evidence path')
            path = bundle / entry['file']
            require(digest(path) == entry['sha256'], 'Environment evidence digest mismatch')
            environment[name] = json.loads(path.read_text(encoding='utf-8'))
        proof, results = environment['manifest'], environment['tests']
        require(proof['format'] == 'integrationer.verified-wasm-fixture.v1' and proof['test_only'] is True and proof['publishable'] is False, 'Invalid probe role')
        require(proof['source_commit'] == results['source_commit'] == expected_commit, 'Probe source mismatch')
        require(proof['source_dirty'] == results['source_dirty'] == manifest['source_dirty'], 'Probe dirty status mismatch')
        require(proof['runtime_image_id'] == manifest['runtime_image_id'], 'Different probe runtime')
        require(proof['tests']['sha256'] == digest(bundle / 'environment-tests.json'), 'Probe results mismatch')
        require(results['artifact_sha256'] == proof['artifact']['sha256'] != expected_artifact, 'Probe/candidate identities must differ')
        require(results['status'] == 'passed' and all(case['status'] == 'passed' for case in results['cases']), 'Probe failure')
        require({case['scenario'] for case in results['cases']} == {'contract', 'network', 'cpu', 'memory', 'wrong_digest_rejected_before_load'}, 'Missing environment probes')
    require(all(case['status'] == 'passed' for case in tests['cases']), 'Failed case')
    return manifest


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('bundle', type=Path)
    parser.add_argument('--expected-artifact', required=True, help='Expected SHA-256 from an independently selected build')
    parser.add_argument('--expected-commit', required=True)
    parser.add_argument('--candidate', action='store_true', help='Require a harmless candidate bundle; reject diagnostic fixtures')
    args = parser.parse_args()
    check(args.bundle, args.expected_artifact, args.expected_commit, candidate=args.candidate)
    print('Integrity verified; unsigned evidence does not establish trusted provenance or publication permission.')
