<template>
  <n-grid x-gap="12" :cols="3" class="container">
    <n-gi> </n-gi>
    <n-gi>
      <div class="green">
        <h1>Login</h1>
        <n-form :model="model">
          <n-form-item path="age" label="Username">
            <n-input />
          </n-form-item>
          <n-form-item path="age" label="Password">
            <n-input />
          </n-form-item>
          <div style="display: flex; justify-content: center">
            <n-button type="success">Login</n-button>
          </div>
        </n-form>
      </div>
    </n-gi>
    <n-gi> </n-gi>
  </n-grid>
</template>
<script>
export default {
  data () {
    return {
      username:"",
      password: ""
    }
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