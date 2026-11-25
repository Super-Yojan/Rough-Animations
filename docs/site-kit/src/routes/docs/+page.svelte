<script lang="ts">
	import { base } from '#lib/base.ts';
	import { onMount } from 'svelte';
	import { Collapsible, CollapsiblePanel, CollapsibleTrigger } from '@ui-sdk/svelte/collapsible';
	import { Input } from '@ui-sdk/svelte/input';
	import Nav from '#lib/Nav.svelte';
	import nav from '#lib/docs-nav.json';
	import { html, toc } from '#lib/docs.ts';

	let query = $state('');
	let active = $state('');
	let mobile = $state(false);
	let menuOpen = $state(false);
	let article: HTMLElement;

	const groups = $derived(
		nav
			.map((g) => ({ ...g, items: g.items.filter((i) => !query || `${i.title} ${i.k}`.toLowerCase().includes(query.toLowerCase())) }))
			.filter((g) => g.items.length)
	);

	onMount(() => {
		const mq = matchMedia('(max-width: 800px)');
		const sync = () => (mobile = mq.matches);
		sync();
		mq.addEventListener('change', sync);
		const script = document.createElement('script');
		script.type = 'module';
		script.src = `${base}/engine/element.js`;
		document.head.append(script);

		article.querySelectorAll<HTMLElement>('.win').forEach((win) => {
			const bar = win.querySelector('.win-bar');
			const code = win.querySelector<HTMLElement>('code[data-raw]');
			if (!bar || !code) return;
			const b = document.createElement('button');
			b.type = 'button';
			b.className = 'copy';
			b.textContent = 'copy';
			b.addEventListener('click', async () => {
				await navigator.clipboard.writeText(code.dataset.raw ?? '');
				b.textContent = 'copied';
				setTimeout(() => (b.textContent = 'copy'), 1200);
			});
			bar.append(b);
		});

		article.querySelector('#live-demo')?.addEventListener('action', (e) => {
			const d = (e as CustomEvent).detail;
			const out = article.querySelector('#live-log');
			if (out) out.textContent = `action event → { action: "${d.action}", id: "${d.id}" }`;
		});

		const spy = new IntersectionObserver(
			(es) => es.forEach((e) => e.isIntersecting && (active = e.target.id)),
			{ rootMargin: '-80px 0px -75% 0px' }
		);
		article.querySelectorAll('h2[id]').forEach((h) => spy.observe(h));
		return () => {
			spy.disconnect();
			mq.removeEventListener('change', sync);
		};
	});
</script>

<svelte:head><title>Docs · Rough-Animations</title></svelte:head>

{#snippet links()}
	<Input class="search" type="search" placeholder="Search the docs…" aria-label="Search the docs" bind:value={query} />
	{#each groups as g (g.title)}
		<h4>{g.title}</h4>
		{#each g.items as i (i.id)}
			<a href={`#${i.id}`} class:on={active === i.id} aria-current={active === i.id ? 'location' : undefined} onclick={() => (menuOpen = false)}>{i.title}</a>
		{/each}
	{/each}
{/snippet}

<Nav current="docs" />

<div class="docs">
	<nav class="side" aria-label="Documentation">
		{#if mobile}
			<Collapsible bind:open={menuOpen}>
				<CollapsibleTrigger class="btn ghost contents-toggle">Contents</CollapsibleTrigger>
				<CollapsiblePanel>{@render links()}</CollapsiblePanel>
			</Collapsible>
		{:else}
			{@render links()}
		{/if}
	</nav>

	<article class="main" bind:this={article}>
		<h1>Documentation</h1>
		<p class="sub">Everything you need to script hand-drawn 3D and 2D animations in Rust.</p>
		{@html html}
	</article>

	<aside class="toc" aria-label="On this page">
		<div>
			<h4>On this page</h4>
			{#each toc as t (t.id)}
				<a href={`#${t.id}`} class:on={active === t.id}>{t.title}</a>
			{/each}
		</div>
	</aside>
</div>
