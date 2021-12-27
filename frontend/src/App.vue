<template>
  <n-button>naive-ui</n-button>
   <n-form
    inline
    :label-width="80"
    :model="formValue"
    :rules="rules"
    :size="size"
    ref="formRef"
  >
    <n-form-item label="Name" path="user.name">
      <n-input  placeholder="Input Name" />
    </n-form-item>
    <n-form-item label="Age" path="user.age">
      <n-input placeholder="Input Age"  />
    </n-form-item>
    <n-form-item label="Phone" path="phone">
      <n-input placeholder="Phone Number"  />
    </n-form-item>
    <n-form-item>
      <!-- <n-button @click="handleValidateClick">Validate</n-button> -->
    </n-form-item>
  </n-form>
</template>

<script>
export default {
  name: 'App',
}
</script>
