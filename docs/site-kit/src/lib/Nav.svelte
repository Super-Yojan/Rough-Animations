<script lang="ts">
	import { base } from '#lib/base.ts';
	import { Button } from '@ui-sdk/svelte/button';
	import { REPO } from '#lib/content.ts';

	let { current = '' }: { current?: 'docs' | 'catalog' | '' } = $props();
</script>

{#snippet link(href: string, text: string, cls: string, active = false)}
	<Button class={cls}>
		{#snippet render({ props })}<a {...props} href={href.startsWith('/') ? base + href : href} aria-current={active ? 'page' : undefined}>{text}</a>{/snippet}
	</Button>
{/snippet}

<header class="nav">
	<nav class="nav-in" aria-label="Main">
		<a class="brand" href={`${base}/`}><i></i>Rough-Animations</a>
		<span class="hide-sm">{@render link('/#features', 'Features', 'btn nav-link')}</span>
		<span class="hide-sm">{@render link('/#gallery', 'Gallery', 'btn nav-link')}</span>
		<span class="hide-xs">{@render link('/catalog', 'Library', 'btn nav-link', current === 'catalog')}</span>
		{@render link('/docs', 'Docs', 'btn nav-link', current === 'docs')}
		<span class="sp"></span>
		<span class="hide-sm">{@render link(REPO, 'GitHub', 'btn nav-link')}</span>
		{@render link('/docs#quick-start', 'Get started', 'btn primary')}
	</nav>
</header>
