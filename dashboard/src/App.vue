<template>
  <n-message-provider>
    <suspense>
      <template #default>
        <router-view />
      </template>
      <template #fallback>
        <h1>Loading</h1>
      </template>
    </suspense>
  </n-message-provider>
</template>

<script>
import Axios from "axios";
export default {
  setup() {
    // set axios config
  },
  name: "App",
};
</script>
