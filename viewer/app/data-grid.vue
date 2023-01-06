
<script>
'use strict';

const ROW_HEIGHT = 25;
const VIRTUAL_ROWS = 160;

var ViewDataGrid = {
	name: 'view-data-grid',
	data() {
		return {
			row_start: 0,
		};
	},
	props: [
		'desc',
		'data',
	],
	computed: {
		pad_top() {
			return (this.row_start * ROW_HEIGHT) + 'px';
		},
		pad_bottom() {
			let end = Math.max(0, this.desc.data_shape[0] - (this.row_start + VIRTUAL_ROWS));
			return (end * ROW_HEIGHT) + 'px';
		},
		rows() {
			let start = this.row_start;
			let end = Math.min(start + VIRTUAL_ROWS, this.desc.data_shape[0]);
			let rows = [];
			for (let i = start; i < end; i += 1) {
				rows.push(i);
			}
			return rows;
		},
	},
	watch: {
		desc(oldv, newv) {
			this.row_start = 0;
			let container = this.$refs.container;
			if (container) {
				container.scrollTop = 0;
			}
		},
	},
	methods: {
		scrolled(e) {
			let table = this.$refs.table;
			if (table) {
				this.update(table.getBoundingClientRect());
			}
		},
		update(rc) {
			let n = Math.max(0, Math.floor(-rc.y / ROW_HEIGHT) - 10);
			this.row_start = n;
		},
	},
	template: '#view-data-grid',
};
</script>

<template id="view-data-grid">
	<div class="view-data-grid thin-sb" @scroll.passive="scrolled" ref="container">
		<table v-if="desc.data_shape.length == 0" class="mono" ref="table">
			<tr>
				<td>{{ data[0] }}</td>
			</tr>
		</table>
		<table v-else-if="desc.data_shape.length == 1" class="mono" ref="table">
			<tbody>
				<tr :style="{ height: pad_top }"></tr>
				<tr v-for="i in rows" :key="i">
					<th>{{ i }}</th>
					<td>{{ data[i] }}</td>
				</tr>
				<tr :style="{ height: pad_bottom }"></tr>
			</tbody>
		</table>
		<table v-else-if="desc.data_shape.length == 2" class="mono" ref="table">
			<thead>
				<tr>
					<th></th>
					<th v-for="j in desc.data_shape[1]" :key="j">{{ j - 1 }}</th>
				</tr>
			</thead>
			<tbody>
				<tr :style="{ height: pad_top }"></tr>
				<tr v-for="i in rows" :key="i">
					<th>{{ i }}</th>
					<td v-for="j in desc.data_shape[1]" :key="j">{{ data[i * desc.data_shape[1] + (j - 1)] }}</td>
				</tr>
				<tr :style="{ height: pad_bottom }"></tr>
			</tbody>
		</table>
		<div v-else>Not implemented</div>
	</div>
</template>

<style>
.view-data-grid {
	overflow: auto;
}
.view-data-grid table {
	text-align: right;
	border-spacing: 3px;

	margin: 10px;
	border-collapse: separate;
	border-spacing: 0;
}
.view-data-grid table th {
	font-weight: normal;
	user-select: none;
	color: var(--udf-color-description);
	background-color: var(--udf-color-background);
	padding: 2px 6px;
}
.view-data-grid table thead th {
	position: sticky;
	top: 0;
	border-bottom: 1px solid var(--udf-color-border-light);
	padding: 4px 6px;
}
.view-data-grid table td {
	padding: 2px 6px;
}
</style>
