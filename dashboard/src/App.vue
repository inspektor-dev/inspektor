<template>
  <n-message-provider>
    <router-view />
  </n-message-provider>
</template>

<script>
import Axios from "axios";
export default {
  setup() {
    // set axios config
    let token = localStorage.getItem("access-token");
    if (token == null) {
      Axios.defaults.headers["Auth-Token"] = token;
    }
  },
  name: "App",
};
</script>
