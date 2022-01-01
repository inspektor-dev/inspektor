<template>
  <n-grid x-gap="12" :cols="3" class="container">
    <n-gi> </n-gi>
    <n-gi>
      <div>
        <h1>Login</h1>
        <n-form :model="formValue" :rules="rules" ref="formRef">
          <n-form-item path="username" label="Username">
            <n-input
              v-model:value="formValue.username"
              placeholder="Enter username"
            />
          </n-form-item>
          <n-form-item path="password" label="Password">
            <n-input
              v-model:value="formValue.password"
              type="password"
              placeholder="Enter Password"
            />
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
import api from "@/api/api";
import { useRouter } from "vue-router";

export default {
  setup() {
    //console.log("login obj", login)
    let formRef = ref(null);
    const message = useMessage();
    let formValue = ref({
      username: "",
      password: "",
    });
    let router = useRouter();
    return {
      formRef,
      formValue: formValue,
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
        formRef.value.validate(async (errors) => {
          if (!errors) {
            //console.log(formValue.value.username, formValue.value.password);
            try {
              let token = await api.login(
                formValue.value.username,
                formValue.value.password
              );
              localStorage.setItem("access-token", token);
              setTimeout(() => {
                router.push("/dashboard");
              }, 1000);
            } catch (e) {
              console.log(e.response);
              message.error(e.response.data.msg);
            }
          } else {
            console.log(errors);
            message.error("Invalid data");
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