<!--
 Copyright 2022 Balaji (rbalajis25@gmail.com)
 
 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at
 
     http://www.apache.org/licenses/LICENSE-2.0
 
 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
-->

<template>
  <div>
    <n-card title="Microsoft Teams">
      <p>
        Microsoft teams integration lets users to do the temporary approval in
        the teams itself.
      </p>
      <n-button @click="showConfigModal = true">Configure</n-button>
      <n-button style="margin-left: 6px" @click="showSecretToken = true" v-if="showJoin">Join Token</n-button>
      <n-modal v-model:show="showSecretToken">
        <n-card title="Secret Token" style="width: 600px">
          <n-input
            v-model:value="currentSecretToken"
            type="text"
            placeholder="Secret Token"
          />
        </n-card>
      </n-modal>
      <n-modal v-model:show="showConfigModal">
        <n-card
          style="width: 600px"
          title="Configure Cloudwatch"
          :bordered="false"
          size="huge"
        >
          <n-form :model="formValue" ref="formRef" :rules="rules">
            <n-form-item path="appId" label="app id">
              <n-input
                v-model:value="formValue.appId"
                placeholder="app id"
              ></n-input>
            </n-form-item>
            <n-form-item path="appToken" label="app token">
              <n-input
                v-model:value="formValue.appToken"
                placeholder="app token"
              ></n-input>
            </n-form-item>

            <div style="display: flex; justify-content: flex-end">
              <n-button type="success" @click="configureTeams"
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
import { ref, computed } from "vue";
import { useMessage } from "naive-ui";
import { useStore } from "vuex";
import api from "@/api/api";

export default {
  setup() {
    let showConfigModal = ref(false);
    let formValue = ref({
      appId: "",
      appToken: "",
    });
    let formRef = ref(null);
    const message = useMessage();
    let store = useStore();
    let showSecretToken = ref(false)
    return {
      showConfigModal,
      formRef,
      formValue,
      showSecretToken,
      rules: {
        appId: {
          required: true,
          message: "Please enter app id",
          trigger: "blur",
        },
        appToken: {
          required: true,
          message: "Please enter app token",
          trigger: "blur",
        },
      },
      configureTeams: (e) => {
        e.preventDefault();
        formRef.value.validate(async (error) => {
          if (error) {
            message.error("Invalid data");
            return;
          }
          await api.configureTeams(formValue.value);
          await store.dispatch("refreshConfig");
          showConfigModal.value = false;
        });
      },
      showJoin: computed(() => {
        return (
          store.state.config.integrationMeta.isTeamConfigured &&
          !store.state.config.integrationMeta.isTeamAdminConfigured
        );
      }),
      currentSecretToken: computed(() => store.state.config.integrationMeta.teamsJoinToken)
    };
  },
};
</script>