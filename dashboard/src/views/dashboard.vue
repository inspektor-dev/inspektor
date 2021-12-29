<template>
  <div class="about">
    <n-card :bordered="false">
      <n-grid x-gap="12" :cols="2">
        <n-gi>
          <h1>Inspektor dashboard</h1>
        </n-gi>
        <n-gi> <div class="button-pos"><n-button type="error">Logout</n-button></div> </n-gi>
      </n-grid>

      <n-tabs type="line">
        <n-tab-pane name="oasis" tab="Datasource"><datasources></datasources></n-tab-pane>
        <n-tab-pane name="the beatles" tab="Admin"><admin/></n-tab-pane>
        <n-tab-pane name="jay chou" tab="Connections">Qilixiang</n-tab-pane>
      </n-tabs></n-card
    >
  </div>
</template>

<style scoped>
.button-pos{
  position: absolute;
  top: 25%;
  right: 2%;
}
</style>
<script>
import Datasources from  '@/components/Datasources.vue'
import Admin from '@/components/Admin.vue'
export default {
  components: {
    Datasources,
    Admin
  }
}
</script>