<template>
  <n-grid x-gap="12" :cols="3" class="container">
    <n-gi> </n-gi>
    <n-gi>
      <div class="green">
        <h1>Login</h1>
        <n-form :model="formValue" :rules="rules" ref="formRef">
          <n-form-item path="username" label="Username">
            <n-input v-model:value="formValue.username" placeholder="Enter username"/>
          </n-form-item>
          <n-form-item path="password" label="Password">
            <n-input v-model:value="formValue.password" type="password" placeholder="Enter Password"/>
          </n-form-item>
          <div style="display: flex; justify-content: center">
            <n-button type="success" @click="handleValidate">Login</n-button>
          </div>
        </n-form>
      </div>
    </n-gi>
    <n-gi> </n-gi>
  </n-grid>
</template>
<script>
import { ref } from "vue";
import { useMessage } from "naive-ui";
export default {
  setup() {
    let formRef = ref(null);
    const message = useMessage();

    return {
      formRef,
      formValue: ref({
        username: "",
        password: "",
      }),
      rules: {
        username: {
          required: true,
          message: "Please enter your username",
          trigger: "blur",
        },
        password: {
          required: true,
          message: "Please enter your password",
          trigger: "blur",
        },
      },
      handleValidate(e) {
        e.preventDefault();
        formRef.value.validate((errors) => {
          if (!errors) {
             message.warning("Nivetha is a nandu kutty");
            console.log("to");
          } else {
            console.log(errors);
            // message.error("Invalid");
          }
        });
      },
    };
  },

  name: "Home",
};
</script>
<style scoped>
.container {
  height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding-top: 10%;
}
</style>