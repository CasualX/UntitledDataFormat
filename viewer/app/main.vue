
<script>
'use strict';

var udf = null;
import('./deps/udf.mjs').then(udf => (window.udf = udf));

function argv() {
	let result = {};
	for (var arg of window.location.hash.substring(1).split("&")) {
		let pos = arg.indexOf("=");
		if (pos >= 0) {
			result[arg.substring(0, pos)] = decodeURIComponent(arg.substring(pos + 1));
		}
	}
	return result;
}

var AppMain = {
	name: 'app-main',
	data() {
		return {
			reader: null,
			status: "",
		};
	},
	methods: {
		fileInput(e) {
			let files = [...e.target.files];
			// Clears the file input otherwise the same file cannot be submitted again
			e.target.value = null;
			// Expect a single file object
			if (files.length != 1) {
				alert(`Expected a single file, got ${files.length} files`);
				return;
			}
			let file = files[0];
			this.reader = window.udf$reader = new udf.UdfReader(file);
			this.reader.name = file.name;
		},
		dropFile(e) {
			if (this.reader != null) {
				return;
			}
			let files = [...e.dataTransfer.files];
			// Expect a single file object
			if (files.length != 1) {
				alert(`Expected a single file, got ${files.length} files`);
				return;
			}
			let file = files[0];
			this.reader = window.udf$reader = new udf.UdfReader(new udf.BlobIO(file));
			this.reader.name = file.name;
		},
		closeFile(e) {
			this.reader = window.udf$reader = null;
			this.status = "";
		},
		setStatusLine(line) {
			this.status = line;
		},
		async sampleInput(event) {
			let name = event.target.textContent;
			let url = event.target.href;
			let io;
			if (window.location.protocol == 'file:') {
				let response = await fetch(url);
				let blob = await response.blob();
				io = new udf.BlobIO(blob);
				io.size = blob.size;
			}
			else {
				io = new udf.UrlIO(url, {});

				// Get the file size
				let response = await fetch(url, { method: 'HEAD' });
				let accept_ranges = response.headers.get('accept-ranges');
				let content_length = response.headers.get('content-length');
				if (accept_ranges == "bytes" && content_length != null) {
					io.size = parseInt(content_length);
				}
			}
			this.reader = window.udf$reader = new udf.UdfReader(io);
			this.reader.name = name;
		},
	},
	created() {
		// Make dropping files work...
		function preventDefaults(e) {
			e.preventDefault();
		}
		for (let eventName of ['dragenter', 'dragover', 'dragleave', 'drop']) {
			document.body.addEventListener(eventName, preventDefaults);
		}
	},
	template: '#app-main',
};
</script>

<template id="app-main">
	<div class="app-main" @drop="dropFile" @dragenter="">
		<div class="header">
			<div class="logo">
				<svg viewBox="0 0 422 170" xmlns="http://www.w3.org/2000/svg">
					<path
						d="M40.6978 164.435C32.77 161.086 25.6421 156.267 19.5279 150.155C13.4165 144.041 8.61915 136.912 5.25 128.984C1.77246 120.754 0 112.027 0 103.04V0H33.9156V103.04C33.9156 121.078 48.605 135.768 66.6431 135.768C84.6812 135.768 99.3677 121.078 99.3677 103.04V0H133.285V103.04C133.285 112.027 131.512 120.754 128.035 128.984C124.686 136.912 119.87 144.041 113.755 150.155C107.641 156.267 100.513 161.063 92.5854 164.435C84.3559 167.912 75.6284 169.682 66.643 169.682C57.6548 169.682 48.9302 167.912 40.6978 164.435Z"
						fill="currentColor"
					/>
					<path
						d="M154.241 168.712V0H209.54C220.989 0 232.093 2.18115 242.549 6.50244C252.703 10.6919 261.817 16.7197 269.616 24.3882C277.459 32.1006 283.635 41.1299 287.936 51.1963C292.386 61.6304 294.653 72.7544 294.653 84.2476C294.653 95.7393 292.386 106.865 287.936 117.319C283.635 127.409 277.48 136.439 269.637 144.19C261.817 151.904 252.724 157.954 242.569 162.167C232.116 166.509 220.989 168.712 209.54 168.712L154.241 168.712ZM209.54 134.795C223.301 134.795 236.177 129.568 245.833 120.041C255.445 110.558 260.736 97.834 260.736 84.2476C260.736 70.6816 255.445 58.0225 245.854 48.583C236.221 39.1201 223.324 33.9141 209.54 33.9141H188.155V134.795H209.54Z"
						fill="currentColor"
					/>
					<path
						d="M311.178 168.495H345.092V103.257H421.995V71.0698H311.178V168.495ZM311.178 0V32.187H421.995V0H311.178Z"
						fill="currentColor"
					/>
				</svg>
				<span>Viewer</span>
			</div>
			<div></div>
			<div v-if="reader != null" class="fn">{{ reader.name }}</div>
			<div v-else></div>
			<div v-if="reader != null" class="x" @click="closeFile">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					fill="none"
					viewBox="0 0 24 24"
					stroke-width="1.5"
					stroke="currentColor"
				>
					<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
				</svg>
			</div>
			<div v-else></div>
		</div>

		<div class="m" v-if="reader == null">
			<div class="dnd-container">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					fill="none"
					viewBox="0 0 24 24"
					stroke-width="1.5"
					stroke="currentColor"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25M9 16.5v.75m3-3v3M15 12v5.25m-4.5-15H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z"
					/>
				</svg>
				<label for="file-upload" class="link">
					<span>Provide a UDF file</span>
					<input @input="fileInput" id="file-upload" name="file-upload" type="file" class="sr-only" />
				</label>
				<span class="text-sm" style="margin-bottom: 8px">or drag and drop one anywhere on the page.</span>
				<span class="text-sm description">Don't have a UDF file? Try one of the sample files:</span>
				<div>
					<template v-for="(name, index) in ['empty', 'sample', 'bunny', 'seahorse']">
						{{ index != 0 ? ", " : "" }}
						<a class="text-sm" :href="`samples/${name}.udf`" @click.prevent="sampleInput($event)">{{ name }}.udf</a>
					</template>
				</div>
			</div>
		</div>
		<app-viewer v-if="reader != null" :reader="reader" @set-status="setStatusLine"></app-viewer>
		<div class="footer">{{ status }}</div>
	</div>
</template>

<style>
@font-face {
	font-family: 'Inter';
	font-style: normal;
	font-weight: 100 900;
	font-display: swap;
	src: url('./deps/Inter.woff2') format('woff2');
	unicode-range: U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+2000-206F, U+2074, U+20AC,
		U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD;
}

@font-face {
	font-family: 'Roboto Mono';
	font-style: normal;
	font-weight: 100 700;
	font-display: swap;
	src: url('./deps/Roboto Mono.woff2') format('woff2');
	unicode-range: U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+2000-206F, U+2074, U+20AC,
		U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD;
}

:root {
	--udf-links: hsl(213 94% 68%);
	--udf-links-hover: hsl(217 91% 60%);
	--udf-links-underline: hsl(217 91% 60% / 0.3);
	--udf-footer: hsl(221 83% 53% / 0.5);
	--udf-color-off-white: hsl(240 7% 98%);
	--udf-color-light: hsl(240 6% 90%);
	--udf-color-body: hsl(240 5% 65%);
	--udf-color-description: hsl(240 4% 46%);
	--udf-color-border-light: hsl(240 5% 34%);
	--udf-color-background: hsl(240 6% 10%);
	--udf-color-border: hsl(0 0% 100% / 0.1);
	--udf-color-background-highlight: hsl(0 0% 100% / 0.03);
}

html, body, #app, .app-main {
	font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto,
		'Helvetica Neue', 'Noto Sans', Arial, sans-serif, 'Apple Color Emoji', 'Segoe UI Emoji', 'Segoe UI Symbol',
		'Noto Color Emoji';
	font-weight: 350;
	font-size: 16px;
	background-color: var(--udf-color-background);
	color: var(--udf-color-body);
	padding: 0;
	margin: 0;
	width: 100%;
	height: 100%;
	overflow: hidden;
}

.app-main {
	display: grid;
	grid-template: 42px calc(100% - 42px - 32px) 32px / auto;
}

.app-main > .header {
	background-color: var(--udf-color-background-highlight);
	display: grid;
	grid-template: auto / 200px auto min-content 48px;
	align-items: center;
	border-bottom: 1px solid var(--udf-color-border);
}

.app-main > .header > .logo {
	user-select: none;
	padding-left: 12px;
	display: flex;
}
.app-main > .header > .logo > span {
	align-self: flex-end;
	margin-bottom: -4px;
	height: 24px;
	font-weight: 500;
	font-size: 18px;
	color: var(--udf-color-body);
	margin-left: 8px;
}
.app-main > .header > .logo > svg {
	width: 50px;
	color: var(--udf-color-light);
}

.app-main > .header > .fn {
	justify-self: end;
}
.app-main > .header > .x {
	display: flex;
	align-items: center;
	cursor: pointer;
	user-select: none;
	width: 42px;
	padding: 0px 12px;
	color: var(--udf-color-body);
}
.app-main > .header > .x > svg {
	width: 1.25rem;
	height: 1.25rem;
}
.app-main > .header > .x:hover {
	color: white;
}

.app-main > .m {
	background-color: var(--udf-color-background);
	padding: 12px;
}

.app-main > .footer {
	display: flex;
	align-items: center;
	background-color: var(--udf-footer);
	color: #fff;
	padding-left: 12px;
	font-size: 0.875rem;
}

.mono {
	font-family: 'Roboto Mono', ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono',
		'Courier New', monospace;
}

.key {
	color: var(--udf-color-description);
}

a, .link {
	color: var(--udf-links);
	text-decoration: underline transparent;
	cursor: pointer;
}

a:hover, .link:hover {
	color: var(--udf-links-hover);
	text-decoration: underline var(--udf-links-underline);
}

.dnd-container {
	display: flex;
	flex-direction: column;
	box-sizing: border-box;
	width: 100%;
	height: 100%;
	justify-items: center;
	justify-content: center;
	align-items: center;
	border-radius: 0.75rem;
	border: 2px dashed var(--udf-color-border-light);
	padding: 1.5rem;
}

.dnd-container > svg {
	height: 42px;
	width: 42px;
	color: var(--udf-color-description);
	margin-bottom: 0.5rem;
}

.dnd-container > * + * {
	margin-top: 0.5rem;
}

.sr-only {
	position: absolute;
	width: 1px;
	height: 1px;
	padding: 0;
	margin: -1px;
	overflow: hidden;
	clip: rect(0, 0, 0, 0);
	white-space: nowrap;
	border-width: 0;
}
.description {
	color: var(--udf-color-description);
}
.text-sm {
	font-size: 0.875rem;
}

.thin-sb {
	/* Foreground, Background */
	scrollbar-color: var(--udf-color-description) var(--udf-color-background);
	scrollbar-width: thin;
}
.thin-sb::-webkit-scrollbar {
	width: 10px; /* Vertical scrollbars */
	height: 10px; /* Horizontal scrollbars */
}
.thin-sb::-webkit-scrollbar-thumb {
	background: var(--udf-color-description); /* Foreground */
}
.thin-sb::-webkit-scrollbar-track {
	background: var(--udf-color-background); /* Background */
}
</style>
