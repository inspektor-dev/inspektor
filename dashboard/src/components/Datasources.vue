<template >
  <n-button type="success" @click="showModal = true">Add Datasources</n-button>
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
  <div style="padding-top: 2%">
    <n-data-table :columns="columns" :data="data" :pagination="pagination" />
  </div>
</template>
<script>
import { ref, h, computed } from "vue";
import { NTag } from "naive-ui";
import AddDatasource from "./AddDatasource.vue";
import { useStore } from "vuex";

import api from "@/api/api";

const createColumn = () => {
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
  ];
};
export default {
  components: { AddDatasource },
  async setup() {
    let store = useStore();
    await store.dispatch("updateDatasource");
    let showModal = ref(false);
    return {
      showModal: showModal,
      data: computed(() => {
        return store.state.datasources;
      }),
      columns: createColumn(),
      datasourceAdded: async () => {
        showModal.value = false;
        await store.dispatch("updateDatasource");
      },
      count: computed(() => {
        return store.state.count;
      }),
    };
  },
  name: "Datasources",
};
</script>