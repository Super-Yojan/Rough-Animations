<script lang="ts">
	import { Button } from '@ui-sdk/svelte/button';
	import { highlight, type Lang } from '#lib/highlight.ts';

	let { file, code, lang = 'rust', copyable = true }: { file: string; code: string; lang?: Lang; copyable?: boolean } = $props();
	let label = $state('copy');

	async function copy() {
		await navigator.clipboard.writeText(code);
		label = 'copied';
		setTimeout(() => (label = 'copy'), 1200);
	}
</script>

<div class="win">
	<div class="win-bar">
		<i></i><i></i><i></i><span>{file}</span>
		{#if copyable}<Button class="copy" onclick={copy} aria-label={`Copy ${file}`}>{label}</Button>{/if}
	</div>
	<pre><code>{@html highlight(code, lang)}</code></pre>
</div>
