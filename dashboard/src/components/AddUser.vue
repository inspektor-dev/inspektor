<template>
  <div>
    <n-form :model="formValue" :rules="rules" ref="formRef">
      <n-form-item path="username" label="Username">
        <n-input
          v-model:value="formValue.username"
          placeholder="name@company.dev"
        />
      </n-form-item>
      <n-form-item path="password" label="Password">
        <n-input
          v-model:value="formValue.password"
          placeholder="shh!@1.s"
          type="password"
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
        <n-button type="success" @click="addUser">Add User</n-button>
      </div>
    </n-form>
  </div>
</template>

<script>
import { ref, computed } from "vue";
import api from "@/api/api";
import { useMessage } from "naive-ui";

export default {
  setup(_, { emit }) {
    let formRef = ref(null);
    const message = useMessage();
    let formValue = ref({
      username: "",
      password: "",
      roles: [],
    });
    return {
      formRef,
      formValue,
      rules: {
        username: {
          required: true,
          message: "Please enter username",
          trigger: "blur",
        },
        password: {
          required: true,
          message: "Please enter password",
          trigger: "blur",
        },
      },
      validateRoles: computed(() => {
        if (formValue.value.roles.length == 0) {
          return "error";
        }
        return "";
      }),
      feedbackForRolesValidation: computed(() => {
        if (formValue.value.roles.length == 0) {
          return "enter roles for user";
        }
        return "";
      }),
      addUser(e) {
        e.preventDefault();
        formRef.value.validate(async (errors) => {
          if (errors) {
            message.error("invalid data");
            return;
          }
          if (formValue.value.roles.length == 0) {
            message.error("Invalid data");
            return;
          }
          await api.addUser(formValue.value);
          emit("onAdd");
        });
      },
    };
  },
};
</script>