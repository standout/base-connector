#!/usr/bin/env python3
"""Verify a harmless candidate separately from adversarial environment fixtures."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

from verify import ROOT, VERIFY, build_once, canonical, command, image_id, isolated_case, require, sha, write
from check_bundle import check
import hashlib


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--builder-image', default='lg1365-wasm-builder:1.93.1')
    parser.add_argument('--runtime-image', default='lg1365-wasm-runtime:5.0.0')
    parser.add_argument('--allow-dirty', action='store_true')
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    source = command(['git', 'rev-parse', 'HEAD'], cwd=ROOT)
    dirty = bool(command(['git', 'status', '--porcelain'], cwd=ROOT))
    require(not dirty or args.allow_dirty, 'Commit source before final verification')
    builder, runtime = image_id(args.builder_image), image_id(args.runtime_image)
    # Probe bytes never enter the candidate directory. Both suites use the same
    # immutable runtime image and isolation implementation, but different WASM.
    probes = output / 'environment-probes'
    subprocess.run([sys.executable, str(VERIFY / 'verify.py'), '--output', str(probes),
                    '--builder-image', builder, '--runtime-image', runtime]
                   + (['--allow-dirty'] if args.allow_dirty else []), check=True)
    bundle = output / 'candidate'
    bundle.mkdir()
    cases = []
    try:
        build_once(builder, bundle, 'candidate', 'synthetic_echo_candidate')
        wasm = bundle / 'connector.wasm'
        wasm.chmod(0o444)
        digest = sha(wasm)
        contract = {kind: json.loads((VERIFY / 'schemas' / f'{kind}.json').read_text(encoding='utf-8')) for kind in ('input', 'output')}
        contract.update(action='echo_contract', diagnostic_actions=[], wit_version='5.0.0', wit_sha256=sha(ROOT / 'wit/standout-app.wit'))
        write(bundle / 'contract.json', contract)
        contract_digest = hashlib.sha256(canonical(contract)).hexdigest()
        cases.append(isolated_case(runtime, bundle, digest, 'candidate'))
        cases.append(isolated_case(runtime, bundle, '0' * 64, 'candidate', reject_digest=True))
        require(sha(wasm) == digest, 'Candidate bytes changed during tests')
        tests = dict(status='passed', source_commit=source, source_dirty=dirty,
                     artifact_sha256=digest, contract_sha256=contract_digest, cases=cases,
                     scope='candidate only; no network/CPU/memory probes in this WASM')
        write(bundle / 'tests.json', tests)
        for name in ('manifest', 'tests'):
            shutil.copyfile(probes / f'{name}.json', bundle / f'environment-{name}.json')
        manifest = dict(format='integrationer.verified-wasm-candidate.v1',
                        artifact_role='internal_connector_candidate', published=False,
                        origin_verified=False, publication_authorized=False,
                        source_repository='https://github.com/standout/base-connector', source_commit=source, source_dirty=dirty,
                        artifact=dict(file='connector.wasm', sha256=digest, size=wasm.stat().st_size),
                        contract=dict(file='contract.json', sha256=contract_digest),
                        tests=dict(file='tests.json', sha256=sha(bundle / 'tests.json'), status='passed'),
                        environment_probes={name: dict(file=f'environment-{name}.json', sha256=sha(bundle / f'environment-{name}.json')) for name in ('manifest', 'tests')},
                        builder_image_id=builder, runtime_image_id=runtime, app_bridge_version='5.0.0',
                        trust='Unsigned integrity/test evidence, not verified origin or permission to import, publish or promote.')
        write(bundle / 'manifest.json', manifest)
        check(bundle, digest, source, allow_dirty=dirty, candidate=True)
        with tempfile.TemporaryDirectory(prefix='candidate-substitution-') as temporary:
            copied = Path(temporary) / 'bundle'
            shutil.copytree(bundle, copied)
            altered = copied / 'connector.wasm'
            altered.chmod(0o644)
            # Specifically reject substituting the dangerous, successfully tested fixture.
            shutil.copyfile(probes / 'connector.wasm', altered)
            try:
                check(copied, digest, source, allow_dirty=dirty, candidate=True)
            except ValueError as error:
                require(str(error) == 'Artifact digest mismatch', 'Unexpected substitution failure')
            else:
                raise RuntimeError('Diagnostic fixture accepted as candidate')
        print(json.dumps(dict(candidate_manifest=str(bundle / 'manifest.json'), artifact_sha256=digest, source_commit=source)))
    except Exception as error:
        (bundle / 'manifest.json').unlink(missing_ok=True)
        write(bundle / 'tests.json', dict(status='failed', cases=cases, error=str(error)))
        raise


if __name__ == '__main__':
    main()
