<script lang="ts">
	import { base } from '#lib/base.ts';
	import { onMount } from 'svelte';
	import { Button } from '@ui-sdk/svelte/button';
	import { Tabs, TabsList, TabsPanel, TabsTab } from '@ui-sdk/svelte/tabs';
	import Nav from '#lib/Nav.svelte';
	import CodeWindow from '#lib/CodeWindow.svelte';
	import { REPO, agentCode, examples, features, gallery, heroCode, steps, webCode, webFeatures } from '#lib/content.ts';
	import { highlight } from '#lib/highlight.ts';

	const install = `rough-animations = { git = "${REPO}" }`;
	let installed = $state(false);

	async function copyInstall() {
		await navigator.clipboard.writeText(install);
		installed = true;
		setTimeout(() => (installed = false), 1200);
	}

	onMount(() => {
		const script = document.createElement('script');
		script.type = 'module';
		script.src = `${base}/engine/element.js`;
		document.head.append(script);
	});
</script>

<svelte:head>
	<title>Rough-Animations: hand-drawn 3D and 2D animation in Rust</title>
	<meta name="description" content="Script a Stage in a few lines of Rust and render a whiteboard-style mp4, gif or png. Isometric solids, boolean shapes, morphing, groups, themes and handwritten LaTeX." />
</svelte:head>

{#snippet link(href: string, text: string, cls: string)}
	<Button class={cls}>
		{#snippet render({ props })}<a {...props} href={href.startsWith('/') ? base + href : href}>{text}</a>{/snippet}
	</Button>
{/snippet}

<Nav />

<main>
	<section class="hero">
		<span class="eyebrow">for Rust · inspired by manim and rough.js</span>
		<h1>Animations that look <u>sketched</u> on a whiteboard.</h1>
		<p class="lead">Write a short script, get an mp4, gif or png. Isometric solids, flat shapes with boolean operations, morphing, groups, themes and handwritten LaTeX, all behind a small API that fits on one page.</p>
		<div class="cta">
			{@render link('/docs#quick-start', 'Read the quick start →', 'btn primary')}
			{@render link('/docs#cheat-sheet', 'API cheat sheet', 'btn ghost')}
		</div>
		<Button class="install" onclick={copyInstall}>
			<b>Cargo.toml</b> rough-animations = {'{'} git = "…/Super-Yojan/Rough-Animations" {'}'} {installed ? '✓ copied' : '⧉'}
		</Button>
	</section>

	<div class="stage">
		<CodeWindow file="src/main.rs" code={heroCode} />
		<div class="preview">
			<div class="cap">out.mp4 · rendered with the library itself</div>
			<img src={`${base}/hero.gif`} alt="An isometric scene being sketched into place" width="900" height="600" />
		</div>
	</div>

	<section class="blk" id="features">
		<div class="sh"><h2>Everything a whiteboard has</h2><p>Small pieces that combine. Positions are tuples, colours are names, numbers can be ints or floats.</p></div>
		<div class="features">
			{#each features as f (f.title)}
				<div class="feat">
					<div class="ic" style:background={f.color} aria-hidden="true">{f.icon}</div>
					<h3>{f.title}</h3>
					<p>{f.body}</p>
					<div class="mini">{f.code}</div>
				</div>
			{/each}
		</div>
	</section>

	<section class="blk" id="how">
		<div class="sh"><h2>Three steps to a video</h2><p>Add objects, play animations, render. A stage plays once, so build a fresh one per video.</p></div>
		<div class="steps">
			{#each steps as s (s.title)}
				<div class="step">
					<h3>{s.title}</h3>
					<p>{s.body}</p>
					<pre><code>{@html highlight(s.code)}</code></pre>
				</div>
			{/each}
		</div>
	</section>

	<section class="blk">
		<div class="sh"><h2>See the code behind it</h2><p>Each tab is a complete program you can paste into <code>src/main.rs</code>.</p></div>
		<Tabs class="tabs" defaultValue="shapes">
			<TabsList aria-label="Examples">
				{#each examples as e (e.value)}
					<TabsTab value={e.value}>{e.label}</TabsTab>
				{/each}
			</TabsList>
			{#each examples as e (e.value)}
				<TabsPanel value={e.value}>
					<CodeWindow file={e.file} code={e.code} />
					<div class="preview">
						<div class="cap">{e.cap}</div>
						<img src={`${base}/${e.img}`} alt={e.alt} loading="lazy" />
					</div>
				</TabsPanel>
			{/each}
		</Tabs>
	</section>

	<section class="blk" id="web">
		<div class="sh"><h2>Drag it. Click it. Put it on the web.</h2><p>The same engine runs live in the browser as WebAssembly. Turn a figure with your mouse, make its parts clickable, scrub it with the page, export it as SVG.</p></div>
		<div class="stage" style="margin-top:0">
			<CodeWindow file="figure.ts" code={webCode} lang="ts" />
			<div class="preview">
				<div class="cap">live, rendered by the engine in your browser: drag to turn, click the rocket</div>
				<rough-figure figure="rocket" loop aria-label="A rocket on its pad. Drag to turn it."></rough-figure>
			</div>
		</div>
		<div class="features" style="margin-top:28px">
			{#each webFeatures as f (f.title)}
				<div class="feat">
					<div class="ic" style:background={f.color} aria-hidden="true">{f.icon}</div>
					<h3>{f.title}</h3>
					<p>{f.body}</p>
				</div>
			{/each}
		</div>
		<div class="cta" style="margin-top:28px">
			{@render link('/docs#web', 'Read the web guide →', 'btn primary')}
			{@render link('/catalog', 'Browse the figure library', 'btn ghost')}
		</div>
	</section>

	<section class="blk" id="gallery">
		<div class="sh"><h2>Gallery</h2><p>Every image here comes from a script in <code>examples/</code>.</p></div>
		<div class="gallery">
			{#each gallery as [img, caption] (img + caption)}
				<figure><img src={`${base}/${img}`} alt="" loading="lazy" /><figcaption>{caption}</figcaption></figure>
			{/each}
		</div>
	</section>

	<section class="blk">
		<div class="agent">
			<div>
				<h2>Built to be driven by an AI agent</h2>
				<p>You cannot watch a video, but you can sample one. The bundled skill teaches the loop: write a short script, draw a still first, then check the motion from a contact sheet of frames. Install it from the plugin marketplace, or with a one-line script that works for any agent.</p>
				{@render link('/docs#agents', 'Using it with agents →', 'btn light')}
			</div>
			<CodeWindow file="terminal" code={agentCode} lang="sh" copyable={false} />
		</div>
	</section>

	<section class="final">
		<h2>Sketch your first scene in five minutes.</h2>
		<div class="cta">
			{@render link('/docs#quick-start', 'Quick start →', 'btn primary')}
			{@render link('/docs', 'Browse the docs', 'btn ghost')}
		</div>
	</section>
</main>

<footer>Rough-Animations · built on roughr, tiny-skia and i_overlay · font: Patrick Hand (SIL OFL 1.1)</footer>
