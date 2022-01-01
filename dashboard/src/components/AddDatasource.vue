<template>
  <div>
    <n-form :model="formValue" :rules="rules" ref="formRef">
      <n-form-item path="name" label="Datasource name">
        <n-input v-model:value="formValue.name" placeholder="prod postgres" />
      </n-form-item>
      <n-form-item label="Datasource Type" path="type">
        <n-select
          placeholder="Select datasource type"
          :options="generalOptions"
          v-model:value="formValue.type"
        />
      </n-form-item>
      <n-form-item
        path="roles"
        label="roles"
        :validation-status="validateRoles"
        :feedback="feedbackForRolesValidation"
      >
        <n-dynamic-tags v-model:value="formValue.roles" />
      </n-form-item>
      <div style="display: flex; justify-content: flex-end">
        <n-button type="success" @click="createDatasource"
          >Create Datasource</n-button
        >
      </div>
    </n-form>
  </div>
</template>

<script>
import { ref, computed } from "vue";
import { useMessage } from "naive-ui";

export default {
  setup() {
    //let roles = ref(['admin']);
    let formRef = ref(null);
    const message = useMessage();
    let formValue = ref({
      name: "",
      type: "",
      roles: ["admin"],
    });
    return {
      formRef,
      formValue,
      rules: {
        name: {
          required: true,
          message: "please enter datasource name",
          trigger: "blur",
        },
        type: {
          required: true,
          message: "please select datasource type",
          trigger: "blur",
        },
      },
      generalOptions: ["Postgres"].map((v) => ({
        label: v,
        value: v,
      })),
      validateRoles: computed(() => {
        if (formValue.value.roles.length == 0) {
          return "error";
        }
        return undefined;
      }),
      feedbackForRolesValidation: computed(() => {
        if (formValue.value.roles.length == 0) {
          return "enter roles for datasource";
        }
        return "";
      }),
      createDatasource(e) {
        e.preventDefault();
        formRef.value.validate((errors) => {
          if (errors) {
            message.error("Invalid data");
            return;
          }
          if (formValue.value.roles.length == 0) {
            message.error("Enter roles for datasource");
          }
        });
      },
    };
  },
  name: "AddDatasource",
};
</script>