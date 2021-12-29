<template>
  <n-button type="success" @click="showModal = true">Add Datasources</n-button>
  <n-modal v-model:show="showModal">
    <n-card
      style="width: 600px"
      title="Add Datasource"
      :bordered="false"
      size="huge"
    >
      <add-datasource> </add-datasource>
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

const createData = () => {
  return [
    {
      key: 0,
      datasourceName: "prod-databse",
      type: "postgres",
      roles: ["admin", "dev"],
    },
  ];
};

const createColumn = () => {
  return [
    {
      title: "Datasource Name",
      key: "datasourceName",
    },
    {
      title: "Type ",
      key: "type",
    },
    {
      title: "Roles",
      key: "roles",
      render(row) {
        const roles = row.roles.map((key) => {
          return h(
            NTag,
            {
              type: "info",
              style: {
                marginRight: "6px",
              },
            },
            {
              default: () => key,
            }
          );
        });
        return roles;
      },
    },
  ];
};
export default {
  components: { AddDatasource },
  setup() {
    return {
      showModal: ref(false),
      data: createData(),
      columns: createColumn(),
    };
  },
  name: "Datasources",
};
</script>