<template>
  <div class="about">
    <n-card :bordered="false">
      <n-grid x-gap="12" :cols="2">
        <n-gi>
          <h1>Inspektor dashboard</h1>
        </n-gi>
        <n-gi> <div class="button-pos"><n-button type="error" @click="logout">Logout</n-button></div> </n-gi>
      </n-grid>

      <n-tabs type="line">
        <n-tab-pane name="oasis" tab="Datasource"><datasources></datasources></n-tab-pane>
        <n-tab-pane name="the beatles" tab="Admin" v-if="isAdmin"><admin/></n-tab-pane>
      </n-tabs></n-card
    >
  </div>
</template>

<style scoped>
.button-pos{
  position: absolute;
   top: 12%;
  right: 2%; 
}
</style>
<script>
import Datasources from  '@/components/Datasources.vue'
import Admin from '@/components/Admin.vue'
import {useRouter} from 'vue-router';
import {useStore} from 'vuex';
import {computed} from 'vue';
export default {
  async setup() {
    let router = useRouter();
    let store = useStore();
    await store.dispatch("init");
    await store.dispatch("updateDatasource");
    return {
      logout: async () =>{
        localStorage.clear();
        await store.dispatch("reset");
        router.push("/")
      },
      isAdmin: computed(() => store.state.isAdmin)
    }
  },
  components: {
    Datasources,
    Admin
  }
}
</script>