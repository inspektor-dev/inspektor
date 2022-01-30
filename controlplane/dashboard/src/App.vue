<template>
  <n-loading-bar-provider>
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
  </n-loading-bar-provider>
</template>

<script>
//import { useStore } from "vuex";
export default {
  setup() {
    // let store = useStore();
    //await store.dispatch("init");
    // set axios config
  },
  name: "App",
};
</script>
