<template>
  <n-config-provider :theme="theme">

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
  </n-config-provider>
</template>

<script>
import { ref } from 'vue'

/**
 * Use this for type hints under js file
 * @type import('naive-ui').GlobalThemeOverrides
 */
// const themeOverrides = {
//   common:{
//     baseColor: "#000000FF"
//   }
// };

//import { useStore } from "vuex";
export default {
  setup() {
    // let store = useStore();
    //await store.dispatch("init");
    // set axios config
    return {
     // themeOverrides,
      theme: ref(null)
    };
  },
  name: "App",
};
</script>
