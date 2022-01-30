<template>
  <div class="about">
    <n-card :bordered="false">
      <n-grid x-gap="12" :cols="2">
        <n-gi>
          <h1>Inspektor dashboard</h1>
        </n-gi>
        <n-gi>
          <div class="refresh-button-pos" v-if="policyExist">
            <p>Policy hash: {{policyHash}}</p>
            <n-button type="success" @click="refreshPolicy"
              >Refresh Policy</n-button
            >
          </div>
        </n-gi>
        <n-gi class="button-pos">
          <div><n-button type="error" @click="logout">Logout</n-button></div>
        </n-gi>
      </n-grid>

      <n-tabs type="line">
        <n-tab-pane name="oasis" tab="Datasource"
          ><datasources></datasources
        ></n-tab-pane>
        <n-tab-pane name="the beatles" tab="Admin" v-if="isAdmin"
          ><admin
        /></n-tab-pane> </n-tabs
    ></n-card>
  </div>
</template>

<style scoped>
.button-pos {
  position: absolute;
  top: 12%;
  right: 2%;
}
.refresh-button-pos {
  position: absolute;
  top: 3.5%;
  right: 8%;
}
</style>
<script>
import Datasources from "@/components/Datasources.vue";
import Admin from "@/components/Admin.vue";
import { useRouter } from "vue-router";
import { useStore } from "vuex";
import { computed } from "vue";
import { useLoadingBar } from "naive-ui";
import { useMessage } from "naive-ui";
import api from "@/api/api";

export default {
  async setup() {
    let router = useRouter();
    let store = useStore();
    const loadingBar = useLoadingBar();
    const message = useMessage();
    await store.dispatch("init");
    await store.dispatch("updateDatasource");
    return {
      logout: async () => {
        localStorage.clear();
        await store.dispatch("reset");
        router.push("/");
      },
      refreshPolicy: async () => {
        loadingBar.start();
        await api.refreshPolicy();
        await store.dispatch("refreshConfig")
        loadingBar.finish();
        message.success("Policy updated");
      },
      isAdmin: computed(() => store.state.isAdmin),
      policyExist: computed(() => {
        if (store.state.config.policyRepoUrl == undefined) {
          return false;
        }
        return store.state.config.policyRepoUrl != "";
      }),
      policyHash: computed(() =>{
        return store.state.config.policyHash
      })
    };
  },
  components: {
    Datasources,
    Admin,
  },
};
</script>