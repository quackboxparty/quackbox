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
		<p class="answer">✓ {answerText}</p>
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
	}
	.muted {
		color: var(--color-text-muted);
		font-size: calc(0.9rem * var(--font-scale));
	}
	.center {
		text-align: center;
	}
	.answer {
		color: var(--color-success);
		font-weight: 600;
		font-size: calc(1.3rem * var(--font-scale));
		margin: 0;
	}
	.flash {
		text-align: center;
		font-family: var(--font-heading);
		font-size: calc(1.2rem * var(--font-scale));
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
		width: min(24rem, 100%);
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.row {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		padding: var(--space-2) var(--space-3);
		border: var(--border-width) var(--border-style) var(--border-color);
		border-radius: var(--radius-md);
		background: var(--bg-surface);
	}
	.row.you {
		border-color: var(--color-primary);
	}
	.rank {
		font-family: var(--font-heading);
		font-weight: 700;
		color: var(--color-text-muted);
		min-width: 1.5rem;
	}
	.avatar {
		width: 1.75rem;
		height: 1.75rem;
		border-radius: var(--radius-full);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-inverse);
		font-weight: 700;
		font-family: var(--font-heading);
		font-size: calc(0.8rem * var(--font-scale));
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
