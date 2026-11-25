import { base } from './base.ts';
import raw from './docs.html?raw';
import { highlight, type Lang } from './highlight.ts';

const unescape = (s: string) => s.replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"').replace(/&amp;/g, '&');

export const toc = [...raw.matchAll(/<h2 id="([^"]+)">(.*?)<\/h2>/g)].map(([, id, title]) => ({ id, title: unescape(title) }));

export const html = raw
	.replace(/(<img[^>]*?)max-width:760px/g, '$1max-width:100%')
	.replace(/href="\/(?!\/)/g, `href="${base}/`)
	.replace(/<code data-lang="(\w+)">([\s\S]*?)<\/code>/g, (_m, lang: string, body: string) => {
		const l = (lang === 'ts' || lang === 'sh' ? lang : lang === 'rust' ? 'rust' : null) as Lang | null;
		const text = body.replace(/^\n/, '').replace(/\s+$/, '');
		return `<code data-raw="${text.replace(/&/g, '&amp;').replace(/"/g, '&quot;')}">${l ? highlight(unescape(text), l) : text}</code>`;
	})
	.replace(/<pre><code>([\s\S]*?)<\/code>/g, (_m, body: string) => {
		const text = body.replace(/^\n/, '').replace(/\s+$/, '');
		return `<pre><code data-raw="${text.replace(/&/g, '&amp;').replace(/"/g, '&quot;')}">${highlight(unescape(text), 'rust')}</code>`;
	})
	.replace(/<(h[23]) id="([^"]+)">(.*?)<\/\1>/g, '<$1 id="$2">$3<a class="anc" href="#$2" aria-label="Link to this section">#</a></$1>');
