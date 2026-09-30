import { test } from 'node:test';
import assert from 'node:assert/strict';
import type {
  UilabWireGoal as Goal,
  UilabWireGoalStep as GoalStep,
  UilabWireServerMessage as ServerMessage,
  UilabWireStepStatus as StepStatus,
} from '../generated/types.ts';
import { bannerLabel, canStop, goalMessage, isActive, settleGoal, stateLabel, stepViews, STEP_ICONS, stopMessage } from './goal.ts';

function step(status: StepStatus, instruction = `do ${status}`, proposal_id?: string): GoalStep {
  return { instruction, target: 'page:members', why: `why ${instruction}`, status, proposal_id };
}

function goal(over: Partial<Goal> = {}): Goal {
  return { goal_id: 'goal-1', by: 'api-1', text: 'build out the member area', state: 'planning', steps: [], ...over };
}

const running = goal({
  state: 'running',
  current: 1,
  steps: [step('accepted', 'add a list', 'p1'), step('proposed', 'add a card', 'p2'), step('pending', 'add a drawer')],
});

test('a goal message replaces the goal; nothing else touches it', () => {
  const first = goal();
  assert.equal(settleGoal(null, { type: 'goal', value: first }), first);
  assert.equal(settleGoal(first, { type: 'goal', value: running }), running);
  const others: ServerMessage[] = [
    { type: 'refused', value: { check: 'goal_running', message: 'm', by: 'api-1' } },
    { type: 'failed', value: { message: 'm' } },
    { type: 'thinking', value: { target: 'page:members', by: 'api-1' } },
  ];
  for (const msg of others) assert.equal(settleGoal(running, msg), running, msg.type);
  assert.equal(settleGoal(null, others[0]!), null);
});

test('planning and running are active and can be stopped; the rest are over', () => {
  assert.equal(isActive(null), false);
  assert.equal(canStop(null), false);
  for (const state of ['planning', 'running'] as const) {
    assert.equal(isActive(goal({ state })), true, state);
    assert.equal(canStop(goal({ state })), true, state);
  }
  for (const state of ['done', 'stopped', 'failed'] as const) {
    assert.equal(isActive(goal({ state })), false, state);
    assert.equal(canStop(goal({ state })), false, state);
  }
});

test('the banner counts the current step of the steps, and only while the goal is active', () => {
  assert.equal(bannerLabel(running), 'goal 2/3');
  assert.equal(bannerLabel(goal()), 'goal: planning');
  assert.equal(bannerLabel(goal({ ...running, current: 0 })), 'goal 1/3');
  assert.equal(bannerLabel(goal({ ...running, state: 'done', current: undefined })), null);
  assert.equal(bannerLabel(null), null);
});

test('step views carry an icon per status and mark the current step', () => {
  const views = stepViews(running);
  assert.deepEqual(
    views.map((v) => [v.index, v.status, v.current]),
    [
      [0, 'accepted', false],
      [1, 'proposed', true],
      [2, 'pending', false],
    ],
  );
  assert.equal(views[1]!.icon, STEP_ICONS.proposed);
  assert.equal(views[0]!.instruction, 'add a list');
  assert.equal(views[0]!.why, 'why add a list');
  assert.equal(views[0]!.proposal_id, 'p1');
  assert.deepEqual(stepViews(null), []);
  const over = stepViews(goal({ ...running, state: 'stopped', current: undefined }));
  assert.ok(over.every((v) => !v.current), 'nothing is current once the goal is over');
});

test('every status has its own icon', () => {
  const statuses: StepStatus[] = ['pending', 'thinking', 'proposed', 'accepted', 'rejected', 'refused', 'declined'];
  const icons = statuses.map((s) => STEP_ICONS[s]);
  assert.ok(icons.every((i) => typeof i === 'string' && i.length > 0));
  assert.equal(new Set(icons).size, statuses.length);
});

test('the state label says what the goal is doing, and why it failed', () => {
  assert.equal(stateLabel(goal()), 'planning…');
  assert.equal(stateLabel(running), 'running step 2 of 3');
  assert.equal(stateLabel(goal({ state: 'done', steps: running.steps })), 'done');
  assert.equal(stateLabel(goal({ state: 'stopped' })), 'stopped');
  assert.equal(stateLabel(goal({ state: 'failed', message: 'declined: a greeting' })), 'failed: declined: a greeting');
  assert.equal(stateLabel(goal({ state: 'failed' })), 'failed');
});

test('the goal and stop messages carry what was typed and the goal id', () => {
  assert.deepEqual(goalMessage('  build it  ', 'page:members'), { type: 'goal', value: { text: 'build it', target: 'page:members' } });
  assert.deepEqual(goalMessage('build it', undefined), { type: 'goal', value: { text: 'build it' } });
  assert.equal(goalMessage('   ', 'page:members'), null);
  assert.deepEqual(stopMessage(running), { type: 'stop_goal', value: { goal_id: 'goal-1' } });
  assert.equal(stopMessage(goal({ state: 'done' })), null);
  assert.equal(stopMessage(null), null);
});
