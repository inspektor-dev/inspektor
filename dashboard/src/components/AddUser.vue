<template>
  <div>
    <n-form :models="formValue" :rules="rules" ref="formRef">
      <n-form-item path="username" label="Username">
        <n-input
          v-model:value="formValue.username"
          placeholder="name@company.dev"
        />
      </n-form-item>
      <n-form-item path="password" label="Password">
        <n-input
          v-model:value="formValue.username"
          placeholder="shh!@1.s"
          type="password"
        />
      </n-form-item>
      <n-form-item path="roles" label="roles">
        <n-dynamic-tags v-model:value="formValue.roles" />
      </n-form-item>
      <div style="display: flex; justify-content: flex-end">
        <n-button type="success" @click="addUser">Add User</n-button>
      </div>
    </n-form>
  </div>
</template>

<script>
import { ref } from "vue";

export default {
  setup() {
    let formRef = ref(null);
    let formValue = ref({
        username: "",
        password: "",
        roles:[]
    });
    return{
        formRef,
        formValue,
        rules:{}
    }
  },
};
</script>