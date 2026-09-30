<script setup lang="ts">
import { computed } from 'vue';
import type { UilabWireGoal as Goal } from '../generated/types.ts';
import { canStop, stateLabel, stepViews } from '../lib/goal.ts';
import { operatorView, selectNearest, stopGoal } from '../store.ts';

const props = defineProps<{ goal: Goal }>();

const steps = computed(() => stepViews(props.goal));
const label = computed(() => stateLabel(props.goal));
const by = computed(() => operatorView(props.goal.by));
</script>

<template>
  <section class="goal" :class="`goal-${goal.state}`">
    <div class="goal-head">
      <span class="goal-badge">goal</span>
      <span class="goal-state">{{ label }}</span>
      <span v-if="by" class="by small" :style="{ color: by.colour }">
        by {{ by.kind === 'agent' ? '🤖 ' : '' }}{{ by.local ? 'you' : by.name }}
      </span>
      <span class="spacer"></span>
      <button v-if="canStop(goal)" class="goal-stop" @click="stopGoal">Stop</button>
    </div>
    <p class="goal-text">“{{ goal.text }}”</p>
    <ol v-if="steps.length" class="goal-steps">
      <li
        v-for="s in steps"
        :key="s.index"
        class="goal-step"
        :class="[`step-${s.status}`, { current: s.current }]"
        :title="s.why"
      >
        <span class="step-icon" :aria-label="s.status">{{ s.icon }}</span>
        <span class="step-text">{{ s.instruction }}</span>
        <code class="path" @click="selectNearest(s.target)">{{ s.target }}</code>
        <span class="step-status muted small">{{ s.status }}</span>
      </li>
    </ol>
  </section>
</template>
