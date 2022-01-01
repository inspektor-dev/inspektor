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
import { ref, h } from "vue";
import { NTag } from "naive-ui";
import AddDatasource from "./AddDatasource.vue";
import api from "@/api/api";

const createData = () => {
  return [
    // {
    //   key: 0,
    //   datasourceName: "prod-databse",
    //   type: "postgres",
    //   roles: ["admin", "dev"],
    // },
  ];
};

const getDatasources = async () => {
  return await api.getDatasources()
}

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
    console.log("source", await getDatasources())
    let datasources = ref(await getDatasources());
    let showModal = ref(false);
    return {
      showModal: showModal,
      data: datasources,
      columns: createColumn(),
      datasourceAdded: async () => {
        showModal.value = false;
        datasources.value = await getDatasources();
      },
    };
  },
  name: "Datasources",
};
</script>