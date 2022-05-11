<template>
  <n-grid x-gap="12" :cols="4">
    <n-gi span="1">
      <div>
        <n-menu
          :options="menuOptions"
          class="light-green"
          @update:value="handleMenuSelect"
        />
      </div>
    </n-gi>
    <n-gi span="3">
      <div v-if="tagFlag.showUser.value">
        <users />
      </div>
      <div v-if="tagFlag.showIntegration.value">
        <integration/>
      </div>
    </n-gi>
  </n-grid>
  <!-- <n-grid x-gap="2" :cols="2">
    <n-gi span="1">
      
    </n-gi>
    <n-gi span="1">
      <div><h1>hello</h1></div>
    </n-gi>
  </n-grid> -->
</template>

<script>
import {
  AccessibilityOutline as UserIcon,
  GlobeOutline
} from "@vicons/ionicons5";
import { NIcon } from "naive-ui";
import { h, ref } from "vue";
import Users from "./Users.vue";
import Integration from "./Integration.vue";
function renderIcon(icon) {
  return () => h(NIcon, null, { default: () => h(icon) });
}
const menuOptions = [
  {
    label: () => {
      return h("a", {}, "Users");
    },
    key: "showUser",
    icon: renderIcon(UserIcon),
  },
  {
    label: () => {
      return h("a", {}, "3rd Party Integration");
    },
    key: "showIntegration",
    icon: renderIcon(GlobeOutline),
  }
  // {
  //   label: () => {
  //     return h("a", {}, "Github Access");
  //   },
  //   key: "github",
  //   icon: renderIcon(GithubIcon),
  // },
  // {
  //   label: () => {
  //     return h("a", {}, "Slack Integration");
  //   },
  //   key: "slack",
  //   icon: renderIcon(SlackIcon),
  // },
];
export default {
  setup() {
    let tagFlag = {
      showUser: ref(true),
      showIntegration: ref(false)
    }
    return {
      menuOptions: menuOptions,
      handleMenuSelect: (key) => {
        for (const [flagKey, flag] of Object.entries(tagFlag)) {
          if (flagKey == key) {
            flag.value = true;
            continue;
          }
          flag.value = false;
        }
        console.log("steate ", key, tagFlag)
      },
      tagFlag,
    };
  },
  name: "Admin",
  components: {
    Users,
    Integration
  },
};
</script>
<style>
.light-green {
  background-color: rgba(187, 245, 187, 0.12);
}
.green {
  height: 108px;
  background-color: rgba(0, 128, 0, 0.24);
}
</style>