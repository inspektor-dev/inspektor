<template>
  <div>
    <n-grid x-gap="9" :cols="3">
      <n-gi>
        <cloudwatch />
      </n-gi>
    </n-grid>
  </div>
</template>

<script>
import Cloudwatch from "./integration/cloudwatch.vue";
export default {
  components: {
    Cloudwatch,
  },
};
</script>