<template>
  <div class="about">
    <n-card :bordered="false">
      <n-grid x-gap="12" :cols="2">
        <n-gi>
          <h1>Inspektor dashboard</h1>
        </n-gi>
        <n-gi>
          <n-modal v-model:show="showTempCredModal">
            <n-card
              style="width: 600px"
              title="Add Temp Credentials"
              :bordered="false"
              size="huge"
            >
              <add-temp-credentials @onAdd="tempCredAdded">
              </add-temp-credentials>
            </n-card>
          </n-modal>
          <div v-if="isAdmin" class="temp-button-pos">
            <n-button type="success" @click="showTempCredModal = true"
              >Create Temp Credentials</n-button
            >
          </div>
        </n-gi>
        <n-gi>
          <div class="refresh-button-pos" v-if="policyExist">
            <p>Policy hash: {{ policyHash }}</p>
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
        /></n-tab-pane>
          <n-tab-pane name="temp sessions" tab="Temp Sessions" 
          ><temp-sessions
        /></n-tab-pane>
      </n-tabs
    ></n-card>
  </div>
</template>

<style scoped>
.temp-button-pos {
  position: absolute;
  top: 17%;
  right: 15%;
}
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
import AddTempCredentials from "@/components/AddTempCredentials.vue";
import TempSessions from "@/components/TempSessions.vue";
import { useRouter } from "vue-router";
import { useStore } from "vuex";
import { computed } from "vue";
import { useLoadingBar } from "naive-ui";
import { useMessage } from "naive-ui";
import api from "@/api/api";
import { ref } from "vue";

export default {
  async setup() {
    let router = useRouter();
    let store = useStore();
    const loadingBar = useLoadingBar();
    const message = useMessage();
    let showTempCredModal = ref(false);
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
        await store.dispatch("refreshConfig");
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
      policyHash: computed(() => {
        return store.state.config.policyHash;
      }),
      showTempCredModal: showTempCredModal,
      tempCredAdded: () => {
        showTempCredModal.value = false;
      },
    };
  },
  components: {
    Datasources,
    Admin,
    AddTempCredentials,
    TempSessions
  },
};
</script>