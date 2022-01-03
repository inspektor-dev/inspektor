<template>
  <div>
    <n-button type="success" @click="showModal = true">Add Users</n-button>
    <n-modal v-model:show="showModal">
      <n-card
        style="width: 600px"
        title="Add Datasource"
        :bordered="false"
        size="huge"
      >
        <add-user @onAdd="userAdded"> </add-user>
      </n-card>
    </n-modal>
    <n-data-table
      :columns="columns"
      :data="data"
      :pagination="pagination"
      style="padding-top: 2%"
    />
  </div>
</template>

<script>
import { h, ref } from "vue";
import { NTag, NButton } from "naive-ui";
import AddUser from "./AddUser.vue";
const createData = () => {
  return [
    {
      key: 0,
      username: "balaji@secret.com",
      roles: ["admin", "dev"],
    },
  ];
};

const createColumn = () => {
  return [
    {
      title: "User Name",
      key: "username",
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
    {
      title: "Delete",
      key: "roles",
      render() {
        return h(
          NButton,
          {
            type: "error",
            style: {
              marginRight: "6px",
            },
          },
          "Delete"
        );
      },
    },
  ];
};

export default {
  components: { AddUser },
  name: "Users",
  setup: () => {
    let showModal = ref(false);
    return {
      data: createData(),
      columns: createColumn(),
      showModal: showModal,
    };
  },
};
</script>