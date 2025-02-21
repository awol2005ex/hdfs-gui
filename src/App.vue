<script setup lang="ts">
import { attachConsole } from '@tauri-apps/plugin-log';
attachConsole().catch(console.error);
// call detach() if you do not want to print logs to the console anymore
</script>

<template>
  <div>
    	<router-view></router-view>
    </div>
</template>

<style scoped>

</style>