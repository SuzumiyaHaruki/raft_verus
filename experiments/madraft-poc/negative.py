#!/usr/bin/env python3
"""DRAFT: mutate an already checked candidate; SIMULATED edit, actual Rust/Verus checks."""
import json
import os
from pathlib import Path
import re
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))
from orchestrator.config import load_config, profile_by_id
from orchestrator.verifier_runner import verify_profile
from orchestrator.workspace import create_candidate


def early_reply(source):
    signature = re.search(r'pub fn handle_request_vote\(&mut self, (\w+): Message\)', source)
    if not signature:
        raise ValueError('unsupported vote signature for this controlled mutation')
    body = re.search(r'(?m)^    \{\s*$', source[signature.end():])
    if not body:
        raise ValueError('cannot locate the vote method body for this controlled mutation')
    position = signature.end() + body.end()
    request = signature[1]
    defect = f'''
        // SIMULATED DEFECT: send a grant before submitting or completing persistence.
        let before = self.observe();
        return Ok(VoteAction::Reply(Message {{ from: before.id, to: {request}.from,
            term: before.hard.term, rpc: {request}.rpc,
            body: MessageBody::VoteResponse {{ granted: true }} }}));
'''
    return source[:position] + defect + source[position:]


def main():
    output = Path(os.environ.get('CARGO_TARGET_DIR', str(ROOT/'target/madraft-negative'))).resolve().parent/'negative-control'
    output.mkdir(parents=True, exist_ok=True)
    relative = Path('crates/raft-verified/src/madraft.rs')
    original = (ROOT/relative).read_text()
    records = []

    def save(status, reason):
        (output/'summary.json').write_text(json.dumps({
            'status': status, 'reason': reason, 'generation_kind': 'SIMULATED',
            'check_evidence_kind': 'REAL_LOCAL', 'phases': records,
        }, indent=2) + '\n')
        print(status + ': ' + reason + '; artifacts=' + str(output))

    with tempfile.TemporaryDirectory(prefix='raft-madraft-negative-') as tmp:
        candidate = create_candidate(ROOT, Path(tmp)/'candidate')
        config = load_config(candidate)

        def check(phase):
            runtime = verify_profile(candidate, profile_by_id(config, 'madraft-poc-runtime'), output/phase/'runtime')
            proof = verify_profile(candidate, profile_by_id(config, 'madraft-poc-verus'), output/phase/'proof')
            records.append({'phase': phase, 'runtime': runtime, 'proof': proof})
            return runtime, proof

        runtime, proof = check('prerequisite')
        if runtime['result'] != 'PASS' or proof['result'] != 'PASS':
            save('BLOCKED', 'a generated vote implementation must first pass actual behavior and proof checks; no solution is supplied')
            return 2
        try:
            wrong = early_reply(original)
        except ValueError as error:
            save('BLOCKED', str(error))
            return 2
        for phase, body in [('defect', wrong), ('repair', original)]:
            (candidate/relative).write_text(body)
            (output/(phase+'.rs')).write_text(body)
            runtime, proof = check(phase)
            expected = 'FAIL' if phase == 'defect' else 'PASS'
            if runtime['result'] != expected or proof['result'] != expected:
                save('FAIL', phase + ' did not produce the required independent results')
                return 1
            if phase == 'defect':
                logs = '\n'.join(Path(p).read_text() for p in runtime['evidence_paths'])
                if ('test result: FAILED' not in logs or 'persistence_barrier' not in logs
                        or proof.get('verification_summary', {}).get('errors', 0) < 1):
                    save('FAIL', 'a compiler/tool failure is not a demonstrated behavioral/proof rejection')
                    return 1
    assert (ROOT/relative).read_text() == original, 'source must remain untouched'
    save('PASS', 'the controlled early grant was rejected and restoring this candidate repaired it')
    print('NEGATIVE_CONTROL_PASS selected=4 generation=SIMULATED checking=REAL_LOCAL')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
