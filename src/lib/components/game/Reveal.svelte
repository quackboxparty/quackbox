<script lang="ts">
	import type { JudgmentView, PlayerView, QuestionView } from '$lib/bindings/Protocol';
	import type { Verdict } from '$lib/bindings/Verdict';
	import { playerColor, playerInitial, sortedByScore } from '$lib/playerUi';
	import { room, has } from '$lib/room.svelte';
	import { m } from '$lib/paraglide/messages';
	import { correctnessText } from './correctness';
	import QuestionPrompt from './QuestionPrompt.svelte';
	import Button from '$lib/components/Button.svelte';

	let {
		question,
		players,
		judgmentLog
	}: {
		question: QuestionView | null;
		players: Record<string, PlayerView>;
		judgmentLog: JudgmentView[];
	} = $props();

	const answerText = $derived(
		question?.answer ? correctnessText(question.answer, question.variant) : null
	);
	// ponytail: flashes only the last log entry. TODO: group judgments by the
	// current question_id and show every ruling for it (steals, revisions)
	// once reveal needs the full per-question breakdown.
	const lastJudgment = $derived<JudgmentView | null>(judgmentLog.at(-1) ?? null);
	const standings = $derived(sortedByScore(players));

	function verdictLabel(verdict: Exclude<Verdict, 'pending'>): string {
		switch (verdict) {
			case 'correct':
				return m.verdict_correct();
			case 'incorrect':
				return m.verdict_incorrect();
			case 'void':
				return m.verdict_void();
			default:
				return verdict satisfies never;
		}
	}
</script>

<section class="reveal">
	{#if question}
		<QuestionPrompt {question} />
	{/if}
	{#if answerText}
		<div class="answer-card">
			<span class="answer-check">✓</span>
			<span class="answer-text">{answerText}</span>
		</div>
	{/if}
	{#if question?.answer?.explanation}
		<p class="muted center">{question.answer.explanation}</p>
	{/if}

	{#if lastJudgment && lastJudgment.verdict !== 'pending'}
		<p class="flash" data-verdict={lastJudgment.verdict}>
			{lastJudgment.player} — {verdictLabel(lastJudgment.verdict)}
			({lastJudgment.points >= 0 ? '+' : ''}{lastJudgment.points})
		</p>
	{/if}

	<ol class="standings">
		{#each standings as [name, p], i (name)}
			<li class="row" class:you={name === room.player}>
				<span class="rank">{i + 1}</span>
				<span class="avatar" style:background={playerColor(name)}>{playerInitial(name)}</span>
				<span class="name">{name}</span>
				<span class="score">{p.score}</span>
			</li>
		{/each}
	</ol>

	{#if has('Moderate')}
		<Button onclick={() => room.send?.({ kind: 'Next' })}>{m.next()} →</Button>
	{:else}
		<p class="muted center">{m.host_will_advance()}</p>
	{/if}
</section>

<style>
	.reveal {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-6);
		width: 100%;
	}
	.muted {
		color: var(--color-text-muted);
		font-size: clamp(0.95rem, 2cqi, 1.25rem);
	}
	.center {
		text-align: center;
	}
	.answer-card {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-3);
		width: min(60rem, 100%);
		padding: var(--space-6) var(--space-8);
		background: var(--bg-surface-elevated);
		border: var(--border-width) var(--border-style) var(--color-success);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-md);
		color: var(--color-success);
		font-family: var(--font-heading);
		font-weight: 700;
		font-size: clamp(1.4rem, 5cqi, 2.8rem);
		line-height: 1.2;
		text-align: center;
	}
	.answer-check {
		font-size: 1.3em;
	}
	.flash {
		text-align: center;
		font-family: var(--font-heading);
		font-size: clamp(1.1rem, 3cqi, 1.8rem);
		padding: var(--space-3) var(--space-6);
		border-radius: var(--radius-md);
		margin: 0;
		color: var(--color-text-inverse);
	}
	.flash[data-verdict='correct'] {
		background: var(--color-success);
	}
	.flash[data-verdict='incorrect'] {
		background: var(--color-danger);
	}
	.flash[data-verdict='void'] {
		background: var(--color-text-muted);
	}
	.standings {
		list-style: none;
		margin: 0;
		padding: 0;
		width: min(40rem, 100%);
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
	}
	.row {
		display: flex;
		align-items: center;
		gap: var(--space-4);
		padding: var(--space-3) var(--space-4);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
		font-size: clamp(1rem, 2.6cqi, 1.5rem);
	}
	.row.you {
		border-color: var(--color-primary);
	}
	.rank {
		font-family: var(--font-heading);
		font-weight: 700;
		color: var(--color-text-muted);
		min-width: 1.8rem;
	}
	.avatar {
		width: clamp(1.75rem, 5cqi, 2.75rem);
		height: clamp(1.75rem, 5cqi, 2.75rem);
		border-radius: var(--radius-full);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-inverse);
		font-weight: 700;
		font-family: var(--font-heading);
		font-size: clamp(0.8rem, 2.2cqi, 1.2rem);
		flex-shrink: 0;
		text-transform: uppercase;
	}
	.name {
		flex: 1;
	}
	.score {
		font-family: var(--font-heading);
		font-weight: 700;
	}
</style>
