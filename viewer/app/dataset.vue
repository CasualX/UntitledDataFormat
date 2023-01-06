
<script>
'use strict';

var ViewDataset = {
	name: 'view-dataset',
	data() {
		return {
			view_desc: null,
			view_data: null,
		};
	},
	props: [
		'reader',
		'entry',
		'props',
	],
	emits: [
		'read-dataset',
		'set-status',
	],
	computed: {
		viewer_component() {
			if (this.view_desc == null || this.view_data == null) {
				return null;
			}
			return this.getViewerComponent(this.view_desc, this.view_data);
		},
	},
	watch: {
		props(oldv, newv) {
			this.view_desc = window.udf$desc = null;
			this.view_data = window.udf$data = null;
		},
	},
	methods: {
		setStatusLine(line) {
			this.$emit('set-status', line);
		},
		async viewData(desc) {
			try {
				this.setStatusLine(`Reading data ${desc.key_name}...`);
				let data = await this.reader.readData(desc);
				this.setStatusLine(`Reading data ${desc.key_name}... ok`);
				this.view_desc = window.udf$desc = desc;
				this.view_data = window.udf$data = data;
			}
			catch (ex) {
				console.error(ex);
				this.setStatusLine(`Reading data ${desc.key_name}... error: ${ex}`);
			}
		},
		getViewerComponent(desc, data) {
			let { hint, dim, prim } = udf.decodeTypeInfo(desc.type_info);
			if (typeof data == 'string') {
				return 'view-data-text';
			}
			if (hint == udf.TYPE_HINT_JSON || (data && (data.__proto__ == Object.prototype || data.__proto__ == Array.prototype))) {
				return 'view-data-text';
			}
			if (hint == udf.TYPE_HINT_DATASET) {
				return 'view-data-sets';
			}
			if (prim >= udf.TYPE_PRIM_U8 && prim <= udf.TYPE_PRIM_F64) {
				return 'view-data-grid';
			}
			// Unknown type, default to hex viewer
			return null;
		},
		readDataset(fo) {
			this.$emit('read-dataset', fo);
		},
		printTypeInfo(type_info) {
			return udf.printTypeInfo(type_info);
		},
		printFileSize(file_size) {
			return udf.printFileSize(file_size);
		},
		printFileOffset(fo) {
			return udf.printFileOffset(fo);
		},
		printShape(shape) {
			return udf.printShape(shape);
		},
	},
	template: '#view-dataset',
};
</script>

<template id="view-dataset">
	<div class="view-dataset">
		<div class="info thin-sb" :class="{ 'has_viewer': viewer_component != null }">
			<div class="props">
				<div class="key">File offset</div>
				<div class="value">{{ printFileOffset(props.file_offset) }}</div>
				<div class="key">File size</div>
				<div class="value">{{ printFileSize(props.file_offset.size) }}</div>
				<div class="key">Id</div>
				<div class="value">{{ props.id }}</div>
			</div>
			<template v-for="[index, desc] in props.tables.entries()" :key="index">
				<h2 class="key_name">{{ desc.key_name }}</h2>
				<div class="props">
					<div class="key">Type info</div>
					<div class="value">{{ printTypeInfo(desc.type_info) }}</div>
					<template v-if="desc.type_name">
						<div class="key">Type name</div>
						<div class="value">{{ desc.type_name }}</div>
					</template>
					<div class="key">Data size</div>
					<div class="value">{{ printFileSize(desc.data_size) }}</div>
					<div class="key">Data shape</div>
					<div class="value">{{ printShape(desc.data_shape) }}</div>
					<template v-if="desc.index_name">
						<div class="key">Index</div>
						<div class="value">{{ desc.index_name }}</div>
					</template>
					<template v-if="desc.related_name">
						<div class="key">Related</div>
						<div class="value">{{ desc.related_name }}</div>
					</template>
				</div>
				<div v-if="getViewerComponent(desc)" class="link" @click="viewData(desc)">View data</div>
			</template>
		</div>
		<component
			v-if="viewer_component"
			:is="viewer_component"
			:desc="view_desc"
			:data="view_data"
			@read-dataset="readDataset"
		></component>
	</div>
</template>

<style>
.view-dataset {
	display: grid;
	grid-template: auto / 400px auto;
}
.view-dataset > .info {
	padding: 12px;
	overflow-y: auto;
	overflow-x: hidden;
}
.view-dataset > .info:not(.has_viewer) {
	grid-column: span 2;
}
.view-dataset > .info.has_viewer {
	border-right: solid 1px var(--udf-color-border);
}
.view-dataset > .info .key_name {
	color: var(--udf-color-light);
	font-size: 1rem;
	font-weight: 500;
	margin: 24px 0 6px 0;
}
.view-dataset > .info .props {
	display: grid;
	grid-template-columns: 150px auto;
}
.view-dataset > .info .link {
	user-select: none;
}
</style>
