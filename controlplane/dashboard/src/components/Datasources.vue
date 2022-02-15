<template >
  <n-button type="success" v-if="isAdmin" @click="showModal = true"
    >Add Datasources</n-button
  >

  <n-modal v-model:show="showModal">
    <n-card
      style="width: 600px"
      title="Add Datasource"
      :bordered="false"
      size="huge"
    >
      <add-datasource @onAdd="datasourceAdded"> </add-datasource>
    </n-card>
  </n-modal>

  <n-modal v-model:show="showSessionModal">
    <n-card
      style="width: 600px"
      title="Postgres Login Credentials"
      :bordered="false"
      size="huge"
    >
      <session-modal :session="currentSessionMeta" />
    </n-card>
  </n-modal>

  <div style="padding-top: 2%">
    <n-data-table :columns="columns" :data="data" :row-key="rowKey" />
  </div>
</template>
<script>
import { ref, h, computed } from "vue";
import { NButton, NTag } from "naive-ui";
import AddDatasource from "./AddDatasource.vue";
import { useStore } from "vuex";
import { useMessage } from "naive-ui";

import api from "@/api/api";
import SessionModal from "./SessionModal.vue";

const createColumn = (message, showSessionModal, currentSessionMeta, store) => {
  return [
    {
      title: "Datasource Name",
      key: "name",
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
          onClick: async () => {
            try {
              await api.createSession({ datasourceId: row.id });
              await store.dispatch("updateDatasource");
            } catch {
              message.error("Unable to create session");
            }
          },
        };
        let buttonText = "Create Credentials";
        if (row.sessionMeta != undefined) {
          currentSessionMeta.value = row.sessionMeta;
          buttonText = "Show Credentials";
          buttonProperty.onClick = () => {
            showSessionModal.value = true;
          };
        }
        return h(NButton, buttonProperty, buttonText);
      },
    },
    {
      title: "Roles",
      render: (row) => {
        let tags = [];
        for (let i = 0; i < row.roles.length; i++) {
          tags.push(
            h(NTag, { style: { marginLeft: "1px" }, round: true , type: 'info'}, row.roles[i])
          );
        }
        return h("div", {}, tags);
      },
    },
    {
      title: "sidecar token",
      render(row) {
        return h(
          NButton,
          {
            type: "success",
            onClick: () => {
              navigator.clipboard.writeText(row.sidecarToken);
              message.success("Token Copied!!");
            },
          },
          "Copy Token"
        );
      },
    },
  ];
};
export default {
  components: { AddDatasource, SessionModal },
  async setup() {
    let store = useStore();
    let showModal = ref(false);
    let showSessionModal = ref(false);
    let currentSessionMeta = ref({});
    let message = useMessage();
    return {
      currentSessionMeta: currentSessionMeta,
      showModal: showModal,
      showSessionModal: showSessionModal,
      data: ref(computed(() => store.state.datasources)),
      columns: createColumn(
        message,
        showSessionModal,
        currentSessionMeta,
        store
      ),
      datasourceAdded: async () => {
        showModal.value = false;
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
  name: "Datasources",
};
</script>