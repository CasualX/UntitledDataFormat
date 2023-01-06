
<script>
'use strict';

var ViewDataSets = {
	name: 'view-data-sets',
	data() {
		return {};
	},
	props: [
		'desc',
		'data',
	],
	emits: [
		'read-dataset',
	],
	methods: {
		getFileOffset(index, lazy) {
			return {
				offset: Number(this.data[index * 2]),
				size: Number(this.data[index * 2 + 1]),
			};
		},
		printFileOffset(fo) {
			return udf.printFileOffset(fo);
		},
		clickDataset(event, index) {
			let fo = { ...this.getFileOffset(index), lazy: event.ctrlKey };
			this.$emit('read-dataset', fo);
		},
	},
	template: '#view-data-sets',
};
</script>

<template id="view-data-sets">
	<div class="view-data-sets thin-sb">
		<table class="mono">
			<tr v-for="i in data.length / 2">
				<th>{{ i - 1 }}</th>
				<td class="link" @click="clickDataset($event, i - 1)">{{ printFileOffset(getFileOffset(i - 1)) }}</td>
			</tr>
		</table>
	</div>
</template>

<style>
.view-data-sets {
	overflow: auto;
}
.view-data-sets table {
	text-align: right;
	border-spacing: 2px;

	margin: 12px;
	border-collapse: separate;
	border-spacing: 0;
}
.view-data-sets table th {
	font-weight: normal;
	user-select: none;
	color: rgb(150, 150, 150);
	background-color: var(--udf-color-background);
	padding: 0 6px;
}
.view-data-sets table thead th {
	position: sticky;
	top: 0;
}
.view-data-sets table td {
	padding: 0 6px;
}
</style>
