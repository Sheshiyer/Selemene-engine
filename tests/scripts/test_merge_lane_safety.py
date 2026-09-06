import json
import subprocess
from pathlib import Path

import pytest
import yaml


HARNESS = r'''
const fs = require('fs');
const input = JSON.parse(fs.readFileSync(0, 'utf8'));
const scenario = input.scenario;
const basePr = {
  number: 10, title: 'fixture', draft: false, state: 'open',
  head: {ref: 'agent/issue-100-fixture', sha: 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', repo: {full_name: 'owner/repo'}},
  base: {ref: 'main'}, mergeable: scenario.mergeable !== false, mergeable_state: 'clean'
};
const freshPr = Object.assign({}, basePr, scenario.secondHead ? {
  head: {ref: basePr.head.ref, sha: scenario.secondHead, repo: {full_name: 'owner/repo'}}
} : {});
const checks = scenario.checks || [{
  id: 1, name: 'CI Gate', head_sha: 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', app: {id: 15368},
  status: 'completed', conclusion: 'success'
}];
let getCount = 0;
let checkCount = 0;
const merges = [];
const failed = [];
const messages = [];
const summary = {
  addHeading() { return this; },
  addRaw() { return this; },
  addCodeBlock() { return this; },
  write: async function() {}
};
const github = {
  rest: {
    pulls: {
      list: async function() { return {data: [basePr]}; },
      get: async function() { getCount++; return {data: getCount === 1 ? basePr : freshPr}; },
      merge: async function(args) { merges.push(args); return {data: {merged: true}}; }
    },
    issues: {
      get: async function() { return {data: {labels: [{name: 'area:qa'}]}}; }
    },
    checks: {
      listForRef: async function() { return {data: {check_runs: checks}}; }
    }
  },
  paginate: async function(method, args) {
    if (method !== github.rest.checks.listForRef || args.ref !== basePr.head.sha) throw new Error('incorrect check target');
    checkCount++;
    if (scenario.paginateError) throw new Error('API error');
    if (scenario.latePending && checkCount > 1) return [{...checks[0], id: 2, status: 'in_progress', conclusion: null}];
    return checks;
  }
};
const context = {
  repo: {owner: 'owner', repo: 'repo'},
  payload: {inputs: Object.prototype.hasOwnProperty.call(scenario, 'inputs')
    ? scenario.inputs
    : {lane: 'qa-docs', limit: '5', dry_run: 'false'}}
};
const core = {
  notice: function(message) { messages.push(String(message)); },
  setFailed: function(message) { failed.push(String(message)); },
  info: function(message) { messages.push(String(message)); },
  warning: function(message) { messages.push(String(message)); },
  summary
};
(async function() {
  let thrown = null;
  try {
    const AsyncFunction = Object.getPrototypeOf(async function() {}).constructor;
    const run = new AsyncFunction('github', 'context', 'core', input.script);
    await run(github, context, core);
  } catch (error) {
    thrown = String(error && error.message || error);
  }
  process.stdout.write('__RESULT__' + JSON.stringify({merges, failed, messages, thrown}));
})();
'''


SCENARIOS = [
    pytest.param('success', {}, 1, id='success'),
    pytest.param('CI becomes pending', {'latePending': True}, 0, id='late-pending'),
    pytest.param('invalid dry run', {'inputs': {'dry_run': 'garbage'}}, 0, id='invalid-dry-run'),
    pytest.param('missing checks', {'checks': []}, 0, id='missing-checks'),
    pytest.param('wrong app', {'checks': [{'id': 1, 'name': 'CI Gate', 'head_sha': 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'app': {'id': 999}, 'status': 'completed', 'conclusion': 'success'}]}, 0, id='wrong-app'),
    pytest.param('wrong check sha', {'checks': [{'id': 1, 'name': 'CI Gate', 'head_sha': 'other', 'app': {'id': 15368}, 'status': 'completed', 'conclusion': 'success'}]}, 0, id='wrong-check-sha'),
    pytest.param('newer pending', {'checks': [
        {'id': 1, 'name': 'CI Gate', 'head_sha': 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'app': {'id': 15368}, 'status': 'completed', 'conclusion': 'success'},
        {'id': 2, 'name': 'CI Gate', 'head_sha': 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'app': {'id': 15368}, 'status': 'in_progress', 'conclusion': None}
    ]}, 0, id='newer-pending'),
    pytest.param('newer failed', {'checks': [
        {'id': 1, 'name': 'CI Gate', 'head_sha': 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'app': {'id': 15368}, 'status': 'completed', 'conclusion': 'success'},
        {'id': 2, 'name': 'CI Gate', 'head_sha': 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', 'app': {'id': 15368}, 'status': 'completed', 'conclusion': 'failure'}
    ]}, 0, id='newer-failed'),
    pytest.param('second pull changes head', {'secondHead': 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'}, 0, id='second-pull-changes-head'),
    pytest.param('mergeable false', {'mergeable': False}, 0, id='mergeable-false'),
    pytest.param('paginate error', {'paginateError': True}, 0, id='paginate-error'),
    pytest.param('omitted inputs', {'inputs': {}}, 0, id='omitted-inputs'),
    pytest.param('invalid limit', {'inputs': {'lane': 'qa-docs', 'limit': 'bad', 'dry_run': 'false'}}, 0, id='invalid-limit'),
    pytest.param('invalid lane', {'inputs': {'lane': 'not-a-lane', 'limit': '5', 'dry_run': 'false'}}, 0, id='invalid-lane'),
]


def load_script():
    root = Path(__file__).resolve().parents[2]
    workflow = root / '.github' / 'workflows' / 'agent-merge-lane.yml'
    document = yaml.safe_load(workflow.read_text())
    return document['jobs']['merge_lane']['steps'][0]['with']['script']


@pytest.mark.parametrize('name,scenario,expected_merges', SCENARIOS)
def test_merge_lane_safety(name, scenario, expected_merges):
    script = load_script()
    completed = subprocess.run(
        ['node', '-e', HARNESS],
        input=json.dumps({'script': script, 'scenario': scenario}),
        text=True,
        capture_output=True,
        check=False,
        timeout=10,
    )
    assert completed.returncode == 0, completed.stderr
    marker = '__RESULT__'
    assert marker in completed.stdout, completed.stdout
    result = json.loads(completed.stdout.rsplit(marker, 1)[1])
    assert result['thrown'] is None, result['thrown']
    assert len(result['merges']) == expected_merges
    if name == 'success':
        assert result['merges'][-1]['sha'] == 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa'
    if name in {'invalid limit', 'invalid lane', 'invalid dry run'}:
        assert result['failed']
