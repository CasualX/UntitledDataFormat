
<script>
'use strict';

var ViewDataText = {
	name: 'view-data-text',
	props: [
		'desc',
		'data',
	],
	data() {
		return {};
	},
	computed: {
		text() {
			if (typeof this.data == 'string') {
				return this.data;
			}
			return JSON.stringify(this.data, null, 4);
		},
	},
	template: '#view-data-text',
};
</script>

<template id="view-data-text">
	<div class="view-data-text">
		<pre class="mono" readonly="readonly">{{ text }}</pre>
	</div>
</template>

<style>
.view-data-text {
	overflow: auto;
	padding: 12px;
}
</style>
