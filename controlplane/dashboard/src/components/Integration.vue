<template>
  <div>
    <n-grid x-gap="9" :cols="3">
      <n-gi>
        <cloudwatch />
      </n-gi>
      <n-gi>
        <auditlog />
      </n-gi>
      <n-gi>
        <teams/>
      </n-gi>
    </n-grid>
  </div>
</template>

<script>
import Cloudwatch from "./integration/cloudwatch.vue";
import Auditlog from "./integration/auditlog.vue";
import Teams from "./integration/teams.vue";
export default {
  components: {
    Cloudwatch,
    Auditlog,
    Teams,
  },
};
</script>