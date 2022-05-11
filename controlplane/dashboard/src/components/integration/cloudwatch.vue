<template>
  <div>
    <n-card title="Cloud Watch">
      <p>
        Cloud Watch is a log service from AWS. You can configure to send audit
        logs to cloud watch.
      </p>
      <n-button @click="showModal = true">Configure</n-button>
      <n-modal v-model:show="showModal">
        <n-card
          style="width: 600px"
          title="Configure Cloudwatch"
          :bordered="false"
          size="huge"
        >
          <n-form :model="formValue" ref="formRef">
            <n-form-item path="credType" label="Credentials Type">
              <n-select
                v-model:value="formValue.credType"
                :options="credOptions"
                placeholder="Select credentials type"
              />
            </n-form-item>
            <n-form-item path="regionName" label="Region Name">
              <n-select
                v-model:value="formValue.regionName"
                :options="regionOptions"
                placeholder="Select region name"
              />
            </n-form-item>
            <n-form-item path="accessKey" label="Access Key">
              <n-input
                v-model:value="formValue.accessKey"
                placeholder="*****"
              />
            </n-form-item>
            <n-form-item path="secretKey" label="Secret Key">
              <n-input
                v-model:value="formValue.secretKey"
                placeholder="*****"
              />
            </n-form-item>
            <n-form-item path="logGroupName" label="Log Group Name">
              <n-input
                v-model:value="formValue.logGroupName"
                placeholder="Destination Log Group name"
              />
            </n-form-item>
            <n-form-item path="logStreamName" label="Log Stream Name">
              <n-input
                v-model:value="formValue.logStreamName"
                placeholder="Destination Log Stream name"
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
export default {
  setup() {
    let formValue = ref({
      credType: "",
      regionName: "",
      accessKey: "",
      secretKey: "",
      logGroupName: "",
      logStreamName: "",
    });
    let formRef = ref(null);
    return {
      showModal: ref(false),
      formRef,
      formValue,
      regionOptions: [
        "us-east-2",
        "us-east-1",
        "us-west-1",
        "us-west-2",
        "af-south-1",
        "ap-east-1",
        "ap-southeast-3",
        "ap-south-1",
        "ap-northeast-3",
        "ap-northeast-2",
        "ap-southeast-1",
        "ap-southeast-2",
        "ap-northeast-1",
        "ca-central-1",
        "eu-central-1",
        "eu-west-1",
        "eu-west-2",
        "eu-south-1",
        "eu-west-3",
        "eu-north-1",
        "me-south-1",
        "sa-east-1",
        "us-gov-east-1",
        "us-gov-west-1",
      ].map((v) => ({ label: v, value: v })),
      credOptions: ["env", "cred"].map((v) => ({ label: v, value: v })),
    };
  },
};
</script>