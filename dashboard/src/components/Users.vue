<template>
  <div>
    <n-data-table :columns="columns" :data="data" :pagination="pagination" />
  </div>
</template>

<script>
import {  h } from "vue";
import { NTag, NButton } from "naive-ui";
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
  name: "Users",
  setup: () => {
    return {
      data: createData(),
      columns: createColumn(),
    };
  },
};
</script>