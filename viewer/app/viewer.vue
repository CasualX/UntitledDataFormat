
<script>
'use strict';

var AppViewer = {
	name: 'app-viewer',
	data() {
		return {
			selected_index: "header",
			entries: {},
		};
	},
	props: [
		'reader',
	],
	emits: [
		'set-status',
	],
	computed: {
		selected() {
			return this.entries[this.selected_index];
		},
	},
	methods: {
		openView(entry, lazy) {
			let existing = this.entries[entry.key];
			if (!existing) {
				this.entries[entry.key] = entry;
			}
			if (!lazy) {
				this.selected_index = entry.key;
				if (entry.component == 'view-dataview') {
					window.udf$dataset = entry.props;
				}
			}
		},
		closeView(index) {
			if (index == this.selected_index) {
				this.selected_index = "header";
			}
			delete this.entries[index];
		},
		setStatusLine(line) {
			this.$emit('set-status', line);
		},
		async readDataset(fo) {
			let key = udf.printFileOffset(fo);
			if (this.entries.hasOwnProperty(key)) {
				if (!fo.lazy) {
					this.selected_index = key;
				}
				return;
			}
			try {
				this.setStatusLine(`Reading ${key}...`);
				let dataset = await this.reader.readDatasetHeader(fo);
				this.openView({
					key: key,
					title: key,
					component: 'view-dataset',
					view_desc: null,
					props: dataset,
					x: true,
				}, fo.lazy);
				this.setStatusLine(`Reading ${key}... ok`);
			}
			catch (ex) {
				console.error(`Reading ${key}`, ex);
				this.setStatusLine(`Reading ${key}... error: ${ex}`);
			}
		},
	},
	async created() {
		this.setStatusLine(`Reading header...`);
		try {
			let header = window.udf$header = await this.reader.readFileHeader();
			this.openView({
				key: "header",
				title: "Header",
				component: 'view-header',
				props: header,
				x: false,
			});
			this.setStatusLine(`Reading header... ok`);
		}
		catch (ex) {
			console.error("Reading header", ex);
			this.setStatusLine(`Reading header... error: ${ex}`);
		}
	},
	template: '#app-viewer',
};
</script>

<template id="app-viewer">
	<div class="app-viewer">
		<div class="entries mono thin-sb">
			<div
				v-for="[index, entry] in Object.entries(entries)"
				:key="entry.key"
				@click="selected_index = index"
				:class="{'active': selected_index == index}"
			>
				<div class="title" :class="{ 'not_x': !entry.x}">{{ entry.title }}</div>
				<div v-if="entry.x" class="x" @click="closeView(index)">
					<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
						<path
							d="M6.28 5.22a.75.75 0 00-1.06 1.06L8.94 10l-3.72 3.72a.75.75 0 101.06 1.06L10 11.06l3.72 3.72a.75.75 0 101.06-1.06L11.06 10l3.72-3.72a.75.75 0 00-1.06-1.06L10 8.94 6.28 5.22z"
						/>
					</svg>
				</div>
			</div>
		</div>
		<div class="content">
			<component
				v-if="selected"
				:is="selected.component"
				:reader="reader"
				:entry="selected"
				:props="selected.props"
				@read-dataset="readDataset"
				@set-status="setStatusLine"
			></component>
			<template v-else>Placeholder</template>
		</div>
	</div>
</template>

<style>
.app-viewer {
	display: grid;
	grid-template: 100% / 216px auto;
}
.app-viewer > .entries {
	display: flex;
	flex-direction: column;
	overflow-y: auto;
	overflow-x: hidden;
	height: auto;
	border-right: 1px solid var(--udf-color-border);
	padding: 12px;
	font-size: 0.875rem;
}
.app-viewer > .entries > div {
	user-select: none;
	cursor: pointer;
	display: grid;
	grid-template: 32px / auto 32px;
	border-radius: 0.5rem;
	margin-bottom: 4px;
}
.app-viewer > .entries > div > .title {
	padding: 6px 12px;
	border-radius: 0.5rem;
	display: flex;
	align-items: center;
	transition: color 100ms;
}

.app-viewer > .entries > div > div:hover {
	color: var(--udf-color-off-white);
}

.app-viewer > .entries > div > .not_x {
	grid-column: span 2;
}
.app-viewer > .entries > div > .x {
	display: none;
	border-radius: 0.5rem;
	transition: color 100ms;
}

.app-viewer > .entries > div > div.x > svg {
	width: 1.25rem;
	height: 1.25rem;
	color: var(--udf-color-description);
}

.app-viewer > .entries > div:hover > .x:hover > svg {
	color: var(--udf-color-off-white);
}

.app-viewer > .entries > div:hover > div.x {
	display: flex;
	align-items: center;
	justify-content: center;
}
.app-viewer > .entries > div:not(.active) > div:hover {
	background-color: var(--udf-color-background-highlight);
}
.app-viewer > .entries > div.active {
	background-color: var(--udf-color-background-highlight);
	color: var(--udf-color-off-white);
}
.app-viewer > .content {
	display: grid;
	grid-template: 100% / 100%;
}
</style>
