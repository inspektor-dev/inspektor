<template>
  <div>
    <n-card title="Stdout Auditlog">
      <p>
        Stdout Auditlog will log all the user logs with specified log prefix.
      </p>
      <n-button @click="showModal = true">Configure</n-button>
      <n-modal v-model:show="showModal">
        <n-card
          style="width: 600px"
          title="Configure Cloudwatch"
          :bordered="false"
          size="huge"
        >
          <n-form :model="formValue" ref="formRef" :rules="rules">
            <n-form-item path="logPrefix" label="log prefix name">
              <n-input
                v-model:value="formValue.logPrefix"
                placeholder="log prefix name"
              />
            </n-form-item>
            <div style="display: flex; justify-content: flex-end">
              <n-button type="success" @click="configureCloudWatch"
                >Configure</n-button
              >
            </div>
          </n-form>
        </n-card>
      </n-modal>
    </n-card>
  </div>
</template>

<script>
import { ref } from "vue";
import { useMessage } from "naive-ui";
import api from "@/api/api";
export default {
  setup() {
    let formValue = ref({
      logPrefix: ""
    });
    let formRef = ref(null);
    const message = useMessage();
    let showModal = ref(false);
    return {
      showModal,
      formRef,
      formValue,
      rules: {
        logPrefix: {
          required: true,
          message: "Please enter log prefix name",
          trigger: "blur",
        },
      },
      configureCloudWatch: (e) => {
        e.preventDefault();
        formRef.value.validate(async (error) => {
          if (error) {
            message.error("Invalid data");
            return;
          }
          await api.configureAuditLog(formValue.value);
          showModal.value = false;
          message.success("configured")
        });
      },
    };
  },
};
</script>