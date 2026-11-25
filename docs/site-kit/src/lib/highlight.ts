const esc = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
const RUST_KW = /^(use|fn|let|mut|as|pub|struct|enum|impl|for|in|if|else|match|return|const|move|crate|mod|where|dyn|self|Self|true|false)$/;
const TS_KW = /^(import|from|const|let|await|async|export|new|true|false|return|function|type|interface|as|of|null|undefined)$/;

function tokens(src: string, re: RegExp, kw: RegExp, typeCase: boolean) {
	let out = '';
	let last = 0;
	let m: RegExpExecArray | null;
	while ((m = re.exec(src))) {
		out += esc(src.slice(last, m.index));
		const [all, com, str, num, id, call] = m;
		if (com) out += `<span class="c">${esc(com)}</span>`;
		else if (str) out += `<span class="s">${esc(str)}</span>`;
		else if (num) out += `<span class="n">${num}</span>`;
		else if (id && kw.test(id)) out += `<span class="k">${id}</span>${esc(call ?? '')}`;
		else if (id && call) out += `<span class="f">${id}</span>${esc(call)}`;
		else if (id && typeCase && /^[A-Z]/.test(id)) out += `<span class="t">${id}</span>`;
		else out += esc(all);
		last = m.index + all.length;
	}
	return out + esc(src.slice(last));
}

function shell(src: string) {
	return src
		.split('\n')
		.map((l) =>
			l.startsWith('#')
				? `<span class="c">${esc(l)}</span>`
				: esc(l)
						.replace(/^(\$?\s*)(cargo|ffmpeg|cp|git|brew|curl|claude)\b/, '$1<span class="f">$2</span>')
						.replace(/(\s)(--?[a-z][\w-]*)/g, '$1<span class="t">$2</span>')
		)
		.join('\n');
}

export type Lang = 'rust' | 'ts' | 'sh';

export function highlight(src: string, lang: Lang = 'rust') {
	if (lang === 'sh') return shell(src);
	if (lang === 'ts')
		return tokens(
			src,
			/(\/\/[^\n]*)|("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|`(?:[^`\\]|\\.)*`)|(\b\d[\d_.]*\b)|(\b[A-Za-z_$][A-Za-z0-9_$]*\b)(\s*\()?/g,
			TS_KW,
			false
		);
	return tokens(
		src,
		/(\/\/[^\n]*)|("(?:[^"\\]|\\.)*"|r#?"[^"]*"#?)|(\b\d[\d_.]*\b)|(\b[A-Za-z_][A-Za-z0-9_]*\b)(\s*[(!])?|(\?)/g,
		RUST_KW,
		true
	);
}
