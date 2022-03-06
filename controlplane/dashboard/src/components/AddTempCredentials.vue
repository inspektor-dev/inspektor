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
    <n-form :model="formValue" :rules="rules" ref="formRef">
      <n-form-item label="datasources" path="datasourceId">
        <n-select
          placeholder="Select datasource"
          :options="generalOptions"
          v-model:value="formValue.datasourceId"
        />
      </n-form-item>
      <n-form-item label="user" path="userId">
        <n-select
          placeholder="Select User"
          :options="userOptions"
          v-model:value="formValue.userId"
        />
      </n-form-item>
      <n-form-item label="valid expiry minutes" path="expiryMinute">
        <n-input-number
          v-model:value="formValue.expiryMinute"
          placeholder="Enter valid expiry minutes"
        />
      </n-form-item>

      <n-form-item path="roles" label="roles">
        <n-dynamic-tags v-model:value="formValue.roles" />
      </n-form-item>
      <div style="display: flex; justify-content: flex-end">
        <n-button type="success" @click="createTempCredentials"
          >Create Datasource</n-button
        >
      </div>
    </n-form>
  </div>
</template>

<script>
import { ref, computed } from "vue";
import { useMessage } from "naive-ui";
import api from "@/api/api";
import { useStore } from "vuex";

export default {
  async setup(_, { emit }) {
    let store = useStore();
    await store.dispatch("updateUsers");
    //let roles = ref(['admin']);
    let formRef = ref(null);
    const message = useMessage();

    let formValue = ref({
      datasourceId: null,
      userId: null,
      expiryMinute: null,
      roles: [],
    });
    let options = store.state.datasources.map((v) => ({
      label: v.name,
      value: v.id,
    }));
    let userOptions = store.state.users.map((v) => ({
      label: v.username,
      value: v.id,
    }));
    return {
      formRef,
      formValue,
      rules: {
        datasourceId: {
          required: true,
          message: "please select datasource",
          trigger: "blur",
          validator: (_rule, value) => {
            if (value > 0) {
              return true;
            }
            return false;
          },
        },
        userId: {
          required: true,
          message: "please select user",
          trigger: "blur",
          validator: (_rule, value) => {
            console.log("validator user", value);
            if (value > 0) {
              return true;
            }
            return false;
          },
        },
        expiryMinute: {
          required: true,
          message: "Please select expiry minute",
          trigger: "blur",
          validator: (_rule, value) => {
            if (value <= 0) {
              return false;
            }
            return true;
          },
        },
        roles: {
          required: true,
          message: "Please enter role",
          trigger: "blur",
          validator: (_rule, value) => {
            if (value.length == 0) {
              return false;
            }
            return true;
          },
        },
      },
      generalOptions: options,
      userOptions: userOptions,
      createTempCredentials(e) {
        e.preventDefault();
        formRef.value.validate(async (errors) => {
          if (errors) {
            console.log(errors);
            message.error("Invalid data");
            return;
          }
          await api.createTempCredentials(formValue.value);
          
          emit('onAdd')
        });
      },
    };
  },
  name: "AddTempCredentials",
};
</script>