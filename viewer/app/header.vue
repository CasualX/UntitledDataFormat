
<script>
'use strict';

var ViewHeader = {
	name: 'view-header',
	data() {
		return {};
	},
	props: [
		'reader',
		'entry',
		'props',
	],
	emits: [
		'read-dataset',
	],
	methods: {
		readDataset(fo) {
			if (fo.offset != 0) {
				this.$emit('read-dataset', fo);
			}
		},
		printFileSize(file_size) {
			return udf.printFileSize(file_size);
		},
		printFileOffset(fo) {
			return udf.printFileOffset(fo);
		},
	},
	template: '#view-header',
};
</script>

<template id="view-header">
	<div class="view-header">
		<div class="props">
			<div class="key">Magic</div>
			<div class="value">{{ props.magic }}</div>
			<div class="key">File size</div>
			<div class="value">{{ printFileSize(reader.io.size) }}</div>
			<div class="key">Id</div>
			<div class="value">{{ props.id }}</div>
			<div class="key">Root</div>
			<div class="value" :class="{ 'link': props.root.offset != 0 }" @click="readDataset(props.root)">
				{{ printFileOffset(props.root) }}
			</div>
		</div>
	</div>
</template>

<style>
.view-header {
	margin: 12px;
}
.view-header > .props {
	display: grid;
	grid-template-columns: 150px auto;
}
</style>
