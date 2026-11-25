<script lang="ts">
	import { onDestroy, onMount, tick } from 'svelte';
	import { Button } from '@ui-sdk/svelte/button';
	import { Dialog, DialogBackdrop, DialogClose, DialogContent, DialogDescription, DialogTitle } from '@ui-sdk/svelte/dialog';
	import { Input } from '@ui-sdk/svelte/input';
	import { SegmentedControl, SegmentedControlIndicator, SegmentedControlItem } from '@ui-sdk/svelte/segmented-control';
	import { Select, SelectContent, SelectItem, SelectLabel, SelectTrigger, SelectValue } from '@ui-sdk/svelte/select';
	import { Tabs, TabsList, TabsPanel, TabsTab } from '@ui-sdk/svelte/tabs';
	import Nav from '#lib/Nav.svelte';
	import { engine } from '#lib/engine.ts';

	type Action = { name: string; label: string; description: string };
	type Fig = { id: string; name: string; title: string; description: string; category: string; tags: string[]; actions: Action[]; spec: object };

	const themes = ['light', 'dark', 'chalkboard', 'blueprint', 'sepia', 'terminal'];

	let library = $state.raw<Fig[]>([]);
	let categories = $state.raw<{ id: string; title: string }[]>([]);
	let FigurePlayer: any;
	let q = $state('');
	let cat = $state('all');
	let theme = $state<string | null>('light');
	let selected = $state.raw<Fig | null>(null);
	let open = $state(false);
	let canvas = $state<HTMLCanvasElement>();
	let player: any = null;
	let last = $state('');
	let hit = $state('');
	const previews = new Map<string, any>();

	const shown = $derived(
		library.filter((f) => {
			if (cat !== 'all' && f.category !== cat) return false;
			const s = q.trim().toLowerCase();
			return !s || [f.title, f.name, f.id, f.description, ...f.tags].join(' ').toLowerCase().includes(s);
		})
	);

	const withTheme = (spec: object) => ({ ...spec, theme });

	onMount(async () => {
		const [e, l] = await Promise.all([engine('engine/index.js'), engine('engine/library/index.js')]);
		FigurePlayer = e.FigurePlayer;
		categories = l.categories;
		library = l.library;
	});
	onDestroy(() => previews.forEach((p) => p.destroy()));

	function preview(node: HTMLCanvasElement, f: Fig) {
		let live = false;
		const io = new IntersectionObserver(
			(es) => {
				if (!es[0].isIntersecting || live) return;
				live = true;
				FigurePlayer.create(node, withTheme(f.spec), { loop: true, loopDelay: 2.2, interactive: false })
					.then((p: any) => previews.set(f.id, p))
					.catch(console.error);
			},
			{ rootMargin: '200px' }
		);
		io.observe(node);
		return {
			destroy() {
				io.disconnect();
				previews.get(f.id)?.destroy();
				previews.delete(f.id);
			}
		};
	}

	$effect(() => {
		const t = theme;
		previews.forEach((p) => p.setTheme?.(t));
		player?.setTheme(t);
	});

	$effect(() => {
		const f = selected;
		const c = canvas;
		if (!f || !c || !FigurePlayer) return;
		let dead = false;
		last = f.actions.length ? 'Click a part of the figure.' : 'Drag to turn it.';
		hit = '';
		FigurePlayer.create(c, withTheme(f.spec), {
			loop: true,
			loopDelay: 2.5,
			pixelRatio: Math.min(devicePixelRatio, 2),
			onAction: ({ action, id }: { action: string; id: string }) => {
				last = `onAction → { action: "${action}", id: "${id}" }`;
				hit = action;
			}
		}).then((p: any) => {
			if (dead) p.destroy();
			else player = p;
		});
		return () => {
			dead = true;
			player?.destroy();
			player = null;
		};
	});

	const camel = (id: string) => id.replace(/-(.)/g, (_, c) => c.toUpperCase());
	const snippets = (f: Fig) => {
		const names = f.actions.map((a) => `"${a.name}"`).join(' | ') || 'never';
		return [
			['React', `import { ${f.name} } from "rough-figure/library/react";\n\nexport function Example() {\n  return (\n    <${f.name}\n      loop\n      size={420}\n      onAction={(e) => {\n        // e.action: ${names}\n        console.log(e.action);\n      }}\n    />\n  );\n}`],
			['HTML element', `<script type="module" src="rough-figure/element"><\/script>\n\n<rough-figure figure="${f.id}" loop></rough-figure>\n\n<script>\n  document.querySelector("rough-figure")\n    .addEventListener("action", (e) => console.log(e.detail.action));\n<\/script>`],
			['TypeScript', `import { FigurePlayer } from "rough-figure";\nimport { ${camel(f.id)} } from "rough-figure/library";\n\nconst player = await FigurePlayer.create(canvas, ${camel(f.id)}.spec, {\n  loop: true,\n  onAction: ({ action }) => console.log(action),\n});`]
		];
	};

	function download(blob: Blob, name: string) {
		const a = document.createElement('a');
		a.href = URL.createObjectURL(blob);
		a.download = name;
		a.click();
		setTimeout(() => URL.revokeObjectURL(a.href), 2000);
	}
	async function exportAs(kind: 'svg' | 'png') {
		if (!player || !selected) return;
		const t = player.time;
		player.seek(player.duration);
		if (kind === 'svg') download(new Blob([player.toSVG()], { type: 'image/svg+xml' }), `${selected.id}.svg`);
		else download(await player.toBlob(), `${selected.id}.png`);
		player.seek(t);
	}
	async function pick(f: Fig) {
		selected = f;
		open = true;
		await tick();
	}
</script>

<svelte:head><title>Figure library · Rough-Animations</title></svelte:head>

<Nav current="catalog" />

<div class="cat-head">
	<h1>Figure library</h1>
	<p>Hand-drawn 3D figures for product UI: empty states, errors, loaders and more. Each one turns when you drag it and reports clicks on its parts.</p>
</div>

<div class="cat-bar">
	<div>
		<Input type="search" placeholder="Search figures…" aria-label="Search figures" bind:value={q} />
		<SegmentedControl aria-label="Category" bind:value={cat}>
			<SegmentedControlIndicator />
			<SegmentedControlItem value="all">All</SegmentedControlItem>
			{#each categories as c (c.id)}
				<SegmentedControlItem value={c.id}>{c.title}</SegmentedControlItem>
			{/each}
		</SegmentedControl>
		<Select items={themes} bind:value={theme}>
			<SelectTrigger aria-label="Theme"><SelectValue /></SelectTrigger>
			<SelectContent>
				{#each themes as t (t)}<SelectItem value={t}>{t}</SelectItem>{/each}
			</SelectContent>
		</Select>
	</div>
</div>

<main class="cat-main">
	<div class="grid">
		{#each shown as f (f.id)}
			<button class="card" type="button" onclick={() => pick(f)}>
				<canvas use:preview={f} aria-hidden="true"></canvas>
				<div class="meta"><h2>{f.title}<small>&lt;{f.name} /&gt;</small></h2><p>{f.description}</p></div>
			</button>
		{/each}
	</div>
	{#if library.length && !shown.length}<p class="empty">No figures match.</p>{/if}
</main>

<Dialog bind:open>
	<DialogBackdrop />
	<DialogContent class="detail">
		{#if selected}
			<div class="d-in">
				<div class="d-fig">
					<canvas bind:this={canvas}></canvas>
					<div class="d-ctl">
						<Button class="btn sm" onclick={() => player?.replay()}>Replay</Button>
						<Button class="btn sm" onclick={() => exportAs('svg')}>Download SVG</Button>
						<Button class="btn sm" onclick={() => exportAs('png')}>Download PNG</Button>
					</div>
					<div class="last" aria-live="polite">{last}</div>
				</div>
				<div class="d-txt">
					<DialogTitle>{selected.title}</DialogTitle>
					<DialogDescription>{selected.description}</DialogDescription>
					<h3>Clickable parts</h3>
					<table>
						<tbody>
							{#each selected.actions as a (a.name)}
								<tr><td class:hit={hit === a.name}>{a.name}</td><td><b>{a.label}.</b> {a.description}</td></tr>
							{:else}
								<tr><td colspan="2">None. This one is just to look at.</td></tr>
							{/each}
						</tbody>
					</table>
					<h3>Use it</h3>
					<Tabs defaultValue="React">
						<TabsList aria-label="Snippet language">
							{#each snippets(selected) as [name] (name)}<TabsTab value={name}>{name}</TabsTab>{/each}
						</TabsList>
						{#each snippets(selected) as [name, code] (name)}
							<TabsPanel value={name}><pre>{code}</pre></TabsPanel>
						{/each}
					</Tabs>
					<DialogClose class="btn sm" style="margin-top:16px">Close</DialogClose>
				</div>
			</div>
		{/if}
	</DialogContent>
</Dialog>
