<!--
 Copyright 2022 Balaji (rbalajis25@gmail.com)
 
 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at
 
     http://www.apache.org/licenses/LICENSE-2.0
 
 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
-->

<template >
  <n-modal v-model:show="showServiceAccountModal">
    <n-card
      style="width: 600px"
      title="Postgres Login Credentials"
      :bordered="false"
      size="huge"
    >
      <session-modal :session="currentSessionMeta" />
    </n-card>
  </n-modal>
  <n-modal v-model:show="showTempCredModal">
    <n-card
      style="width: 600px"
      title="Create Service Account"
      :bordered="false"
      size="huge"
    >
      <add-service-account @onAdd="serviceAccountAdded"> </add-service-account>
    </n-card>
  </n-modal>
  <div v-if="isAdmin" class="temp-button-pos">
    <n-button type="success" @click="showTempCredModal = true"
      >Create Service Account</n-button
    >
  </div>
  <div style="padding-top: 2%">
    <n-data-table :columns="columns" :data="data" :row-key="rowKey" />
  </div>
</template>
<script>
import { ref, h, computed } from "vue";
import { NButton, NTag, NDynamicTags } from "naive-ui";
import { useStore } from "vuex";
import { useMessage } from "naive-ui";
import AddServiceAccount from "@/components/AddServiceAccount.vue";
import api from "@/api/api";
import SessionModal from "./SessionModal.vue";

const createColumn = (message, showServiceAccountModal, currentSessionMeta, store) => {
  return [
    {
      title: "Datasource Name",
      key: "name",
    },
    {
        title: "Service Account Name",
        key: "sessionMeta.meta.serviceAccountName"
    },
    {
      title: "Type",
      key: "type",
    },
    {
      title: "sidecar hostname",
      key: "sidecarHostname",
    },
    {
      title: "Session",
      render(row) {
        let buttonProperty = {
          type: "success",
          onClick: () => {
            showServiceAccountModal.value = true;
          },
        };
        currentSessionMeta.value = row.sessionMeta.meta;
        let buttonText = "Show Credentials";
        return h(NButton, buttonProperty, buttonText);
      },
    },
    {
      title: "Roles",
      render: (row) => {
        // let tags = [];
        // for (let i = 0; i < row.roles.length; i++) {
        //   tags.push(
        //     h(
        //       NTag,
        //       { style: { marginLeft: "1px" }, round: true, type: "info" },
        //       row.roles[i]
        //     )
        //   );
        // }
        // tags.push(
        //   h(
        //     NTag,
        //     { style: { marginLeft: "1px" }, round: true, type: "success" },
        //     "Add Role"
        //   )
        // );
        return h(
          NDynamicTags,
          {
            disabled: true,
            closable: false,
            defaultValue: row.sessionMeta.meta.tempRoles,
          },
          ""
        );
      },
    },
  ];
};
export default {
  components: { SessionModal, AddServiceAccount },
  async setup() {
    let store = useStore();
    let showModal = ref(false);
    let showServiceAccountModal = ref(false);
    let currentSessionMeta = ref({});
    let message = useMessage();
    let showTempCredModal = ref(false);
    return {
      currentSessionMeta: currentSessionMeta,
      showModal: showModal,
      showServiceAccountModal: showServiceAccountModal,
      data: ref(computed(() => store.state.serviceAccountDatasoruces)),
      columns: createColumn(
        message,
        showServiceAccountModal,
        currentSessionMeta,
        store
      ),
      showTempCredModal: showTempCredModal,
      serviceAccountAdded: async () => {
        showTempCredModal.value = false;
         await store.dispatch("updateDatasource");
      },
      count: computed(() => {
        return store.state.count;
      }),
      isAdmin: computed(() => {
        return store.state.isAdmin;
      }),
      rowKey: (data) => data.id,
    };
  },
  name: "TempSessions",
  
};
</script>