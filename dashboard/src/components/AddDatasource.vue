<template>
  <!-- <n-form :model="model">
    <n-form-item-gi label="Datasource Type">
      <n-select placeholder="Select" :options="types" />
    </n-form-item-gi>
  </n-form> -->
  <div>
    <n-form-item path="bane" label="Datasource name">
      <n-input />
    </n-form-item>
    <n-form :model="model">
      <n-form-item label="Datasource Type" path="type">
        <n-select placeholder="Select" :options="generalOptions" />
      </n-form-item>
      <n-form-item path="roles" label="roles">
        <n-dynamic-tags v-model:value="roles" />
      </n-form-item>
      <div style="display: flex; justify-content: flex-end">
        <n-button type="success">Create Datasource</n-button>
      </div>
    </n-form>
  </div>
</template>

<script>
import { ref } from "vue";

export default {
  setup() {
    //let roles = ref(['admin']);
    return {
      generalOptions: ["Postgres"].map((v) => ({
        label: v,
        value: v,
      })),
      //  roles: roles,
      roles: ref(["admin"]),
    };
  },
  name: "AddDatasource",
};
</script>