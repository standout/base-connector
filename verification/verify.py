#!/usr/bin/env python3
"""Build once; verify identical component bytes in constrained, disposable containers."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import time
import uuid

from check_bundle import check

ROOT = Path(__file__).resolve().parents[1]
VERIFY = ROOT / 'verification'


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def command(args, **kwargs):
    return subprocess.run(args, check=True, text=True, capture_output=True, **kwargs).stdout.strip()


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()


def write(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n', encoding='utf-8')


def image_id(tag):
    return command(['docker', 'image', 'inspect', tag, '--format', '{{.Id}}'])


def isolated_case(image, bundle, digest, scenario, reject_digest=False):
    name = 'lg1365-wasm-' + uuid.uuid4().hex[:12]
    memory = 256 if scenario == 'memory' else 768
    deadline = 5 if scenario == 'cpu' else 20
    args = ['docker', 'create', '--name', name, '--network', 'none', '--read-only',
            '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges',
            '--user', '65534:65534', '--pids-limit', '64', '--cpus', '1',
            '--memory', f'{memory}m', '--memory-swap', f'{memory}m',
            '--ulimit', 'cpu=10:12', '--tmpfs', '/tmp:rw,noexec,nosuid,size=64m',
            '--mount', f'type=bind,source={bundle},target=/artifact,readonly',
            '--mount', f'type=bind,source={VERIFY},target=/verification,readonly',
            image, 'timeout', '--signal=KILL', str(deadline),
            'ruby', '/verification/worker.rb', '/artifact/connector.wasm', digest, scenario]
    created = False
    try:
        command(args)
        created = True
        info = json.loads(command(['docker', 'inspect', name]))[0]
        host = info['HostConfig']
        require(host['NetworkMode'] == 'none' and host['ReadonlyRootfs'], 'Isolation/verification invariant failed')
        require(host['CapDrop'] == ['ALL'] and not host['Privileged'], 'Isolation/verification invariant failed')
        require('no-new-privileges' in host['SecurityOpt'], 'Isolation/verification invariant failed')
        require(host['Memory'] == memory * 1024 * 1024 and host['PidsLimit'] == 64, 'Isolation/verification invariant failed')
        require(info['Config']['User'] == '65534:65534', 'Isolation/verification invariant failed')
        require(all(not mount['RW'] for mount in info['Mounts'] if mount['Type'] == 'bind'), 'Isolation/verification invariant failed')
        started = time.monotonic()
        process = subprocess.run(['docker', 'start', '--attach', name], capture_output=True, text=True, timeout=40)
        elapsed = time.monotonic() - started
        state = json.loads(command(['docker', 'inspect', name]))[0]['State']
        lines = [json.loads(line) for line in process.stdout.splitlines() if line.startswith('{')]
        evidence = {'scenario': scenario, 'exit_code': state['ExitCode'], 'oom_killed': state['OOMKilled'],
                    'elapsed_seconds': round(elapsed, 3), 'stdout': lines, 'stderr': process.stderr[-4000:],
                    'isolation': {'network': host['NetworkMode'], 'readonly_root': host['ReadonlyRootfs'],
                                  'memory_bytes': host['Memory'], 'pids_limit': host['PidsLimit'],
                                  'cpu_quota_nanos': host['NanoCpus'], 'security_opt': host['SecurityOpt'],
                                  'cap_drop': host['CapDrop'], 'user': info['Config']['User']}}
        if reject_digest:
            require(state['ExitCode'] == 1 and not state['OOMKilled'], evidence)
            require('Artifact digest mismatch before load' in evidence['stderr'] and not lines, evidence)
            evidence['scenario'] = 'wrong_digest_rejected_before_load'
        elif scenario in ('cpu', 'memory'):
            require(any(x.get('phase') == 'probe_started' and x.get('artifact_sha256') == digest for x in lines), evidence)
            require(state['ExitCode'] == 137, evidence)
            require(state['OOMKilled'] == (scenario == 'memory'), evidence)
            if scenario == 'cpu':
                require(4 <= elapsed < 15, evidence)
        else:
            require(state['ExitCode'] == 0 and lines[-1]['status'] == 'passed', evidence)
            require(lines[-1]['artifact_sha256'] == digest, evidence)
        evidence['status'] = 'passed'
        return evidence
    finally:
        # Remove only this verifier-created disposable container, never user workloads.
        if created:
            subprocess.run(['docker', 'rm', '--force', name], capture_output=True, check=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', type=Path, required=True, help='New output directory; must not exist')
    parser.add_argument('--builder-image', default='lg1365-wasm-builder:1.93.1')
    parser.add_argument('--runtime-image', default='lg1365-wasm-runtime:5.0.0')
    parser.add_argument('--allow-dirty', action='store_true', help='Development only; marks source_dirty=true')
    args = parser.parse_args()
    source_commit = command(['git', 'rev-parse', 'HEAD'], cwd=ROOT)
    dirty = bool(command(['git', 'status', '--porcelain'], cwd=ROOT))
    if dirty and not args.allow_dirty:
        raise SystemExit('Commit the source before producing verified evidence (or use explicit --allow-dirty for development).')
    bundle = args.output.resolve()
    bundle.mkdir(parents=True, exist_ok=False)
    # Image IDs, not moving tags, are used for every build/test in this run.
    builder = image_id(args.builder_image)
    runtime = image_id(args.runtime_image)
    with tempfile.TemporaryDirectory(prefix='lg1365-wasm-build-') as temporary:
        staging = Path(temporary)
        shutil.copytree(VERIFY / 'fixture', staging, dirs_exist_ok=True)
        shutil.copytree(VERIFY / 'schemas', staging / 'schemas')
        (staging / 'wit').mkdir()
        shutil.copyfile(ROOT / 'wit/standout-app.wit', staging / 'wit/standout-app.wit')
        command(['docker', 'run', '--rm', '--network', 'none', '--mount',
                 f'type=bind,source={staging},target=/source,readonly', builder,
                 'cargo', 'fmt', '--', '--check'], timeout=30)
        build_name = 'lg1365-wasm-build-' + uuid.uuid4().hex[:12]
        try:
            result = subprocess.run(['docker', 'run', '--name', build_name, '--rm', '--cpus', '2', '--memory', '2g', '--pids-limit', '128',
                                 '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges',
                                 '--mount', f'type=bind,source={staging},target=/source', builder,
                                 'cargo', 'build', '--locked', '--release', '--target', 'wasm32-wasip2'],
                                capture_output=True, text=True, timeout=600)
        finally:
            subprocess.run(['docker', 'rm', '--force', build_name], capture_output=True)
        (bundle / 'build.log').write_text(result.stdout + result.stderr, encoding='utf-8')
        result.check_returncode()
        shutil.copyfile(staging / 'target/wasm32-wasip2/release/synthetic_contract_fixture.wasm', bundle / 'connector.wasm')
    wasm = bundle / 'connector.wasm'
    wasm.chmod(0o444)
    digest = sha(wasm)
    contract = {kind: json.loads((VERIFY / 'schemas' / f'{kind}.json').read_text(encoding='utf-8')) for kind in ('input', 'output')}
    contract['action'] = 'echo_contract'
    contract['diagnostic_actions'] = ['network_probe', 'cpu_probe', 'memory_probe']
    contract['wit_sha256'] = sha(ROOT / 'wit/standout-app.wit')
    contract['wit_version'] = '5.0.0'
    write(bundle / 'contract.json', contract)
    contract_digest = hashlib.sha256(canonical(contract)).hexdigest()
    cases = []
    try:
        for scenario in ('contract', 'network', 'cpu', 'memory'):
            cases.append(isolated_case(runtime, bundle, digest, scenario))
            require(sha(wasm) == digest, 'Artifact changed between tests')
            write(bundle / 'tests.json', {'status': 'in_progress', 'cases': cases})
            print(f'{scenario}: passed', flush=True)
        cases.append(isolated_case(runtime, bundle, '0' * 64, 'contract', reject_digest=True))
    except Exception as error:
        write(bundle / 'tests.json', {'status': 'failed', 'cases': cases, 'error': str(error)})
        raise
    tests = {'status': 'passed', 'source_commit': source_commit, 'source_dirty': dirty, 'artifact_sha256': digest, 'contract_sha256': contract_digest, 'cases': cases}
    write(bundle / 'tests.json', tests)
    manifest = {'format': 'integrationer.verified-wasm-fixture.v1', 'test_only': True, 'publishable': False,
                'source_repository': 'https://github.com/standout/base-connector', 'source_commit': source_commit,
                'source_dirty': dirty, 'artifact': {'file': 'connector.wasm', 'sha256': digest, 'size': wasm.stat().st_size},
                'contract': {'file': 'contract.json', 'sha256': contract_digest, 'canonicalization': 'UTF-8 JSON sorted keys, no whitespace; array order retained'},
                'tests': {'file': 'tests.json', 'sha256': sha(bundle / 'tests.json'), 'status': 'passed'},
                'builder_image_id': builder, 'runtime_image_id': runtime, 'app_bridge_version': '5.0.0',
                'app_bridge_reviewed_source': 'ecf9f08e9183d033ca42c24c98794397f8a28c98',
                'trust': 'Unsigned integrity evidence only. Digests do not authenticate source, builder, runtime, or results. No release/publication approval.'}
    require(sha(wasm) == digest, 'Isolation/verification invariant failed')
    write(bundle / 'manifest.json', manifest)
    try:
        check(bundle, digest, source_commit, allow_dirty=dirty)
        # Offline substitution test on a copy: never execute tampered bytes.
        with tempfile.TemporaryDirectory(prefix='lg1365-tamper-') as temporary:
            copied = Path(temporary) / 'bundle'
            shutil.copytree(bundle, copied)
            altered = copied / 'connector.wasm'
            altered.chmod(0o644)
            altered.write_bytes(altered.read_bytes() + b'tampered')
            try:
                check(copied, digest, source_commit, allow_dirty=dirty)
            except ValueError as error:
                require(str(error) == 'Artifact digest mismatch', 'Unexpected substitution rejection')
            else:
                raise RuntimeError('Substituted artifact accepted')
    except Exception as error:
        (bundle / 'manifest.json').unlink(missing_ok=True)
        write(bundle / 'tests.json', {'status': 'failed', 'cases': cases, 'error': str(error)})
        raise
    print(json.dumps({'manifest': str(bundle / 'manifest.json'), 'artifact_sha256': digest, 'source_commit': source_commit, 'source_dirty': dirty}), flush=True)


if __name__ == '__main__':
    main()
