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

  <n-modal v-model:show="showSecretToken">
    <n-card title="Secret Token" style="width: 600px">
      <n-input
        v-model:value="currentSecretToken"
        type="text"
        placeholder="Secret Token"
      />
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
import { NButton, NTag, NDynamicTags } from "naive-ui";
import AddDatasource from "./AddDatasource.vue";
import { useStore } from "vuex";
import { useMessage } from "naive-ui";

import api from "@/api/api";
import SessionModal from "./SessionModal.vue";

const createColumn = (
  message,
  showSessionModal,
  currentSessionMeta,
  store,
  currentSecretToken,
  showSecretToken
) => {
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
            disabled: !store.state.isAdmin,
            closable: false,
            defaultValue: row.roles,
            onUpdateValue: async (updatedRoles) => {
              console.log("updated roles", updatedRoles);
              try {
                await api.updateRoles({
                  type: "DATA_SOURCE",
                  roles: updatedRoles,
                  id: row.id,
                });
                await store.dispatch("updateDatasource");
              } catch {
                message.error("error while adding roles");
              }
            },
          },
          ""
        );
      },
    },
    {
      title: "sidecar token",
      render(row) {
        return h(
          NButton,
          {
            type: "info",
            onClick: () => {
              if (window.isSecureContext) {
                navigator.clipboard.writeText(row.sidecarToken);
                message.success("Token Copied!!");
                return;
              }
              console.log(row.sidecarToken);
              currentSecretToken.value = row.sidecarToken;
              showSecretToken.value = true;
            },
          },
          showTokenText()
        );
      },
    },
    {
      title: "Delete datasource",
      render(row) {
        return h(
          NButton,
          {
            type: "error",
            onClick: async () => {
              await api.deleteDatasource({ datasourceId: row.id });
              await store.dispatch("updateDatasource");
            },
          },
          "delete datasource"
        );
      },
    },
  ];
};

function showTokenText() {
  if (window.isSecureContext) {
    return "Copy Token";
  }
  return "Show Token";
}

export default {
  components: { AddDatasource, SessionModal },
  async setup() {
    let store = useStore();
    let showModal = ref(false);
    let showSessionModal = ref(false);
    let currentSessionMeta = ref({});
    let currentSecretToken = ref("");
    let message = useMessage();
    let showSecretToken = ref(false);
    return {
      currentSessionMeta: currentSessionMeta,
      showModal: showModal,
      showSessionModal: showSessionModal,
      data: ref(computed(() => store.state.datasources)),
      columns: createColumn(
        message,
        showSessionModal,
        currentSessionMeta,
        store,
        currentSecretToken,
        showSecretToken
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
      showSecretToken: showSecretToken,
      currentSecretToken: currentSecretToken,
    };
  },
  name: "Datasources",
};
</script>