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
      <div v-if="showUser">
        <users />
      </div>
      <div v-if="showGithub" class="green"></div>
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
  LogoGithub as GithubIcon,
  LogoSlack as SlackIcon,
} from "@vicons/ionicons5";
import { NIcon } from "naive-ui";
import { h, ref } from "vue";
import Users from "./Users.vue";

function renderIcon(icon) {
  return () => h(NIcon, null, { default: () => h(icon) });
}
const menuOptions = [
  {
    label: () => {
      return h("a", {}, "Users");
    },
    key: "user",
    icon: renderIcon(UserIcon),
  },
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
    let showUser = ref(true);
    let showGithub = ref(false);
    return {
      menuOptions: menuOptions,
      handleMenuSelect: (key) => {
          console.log("key val ", key);
        if (key == "user") {
          showUser.value = true;
          showGithub.value = false;
          return;
        }
        if (key == "github") {
          showUser.value = false;
          showGithub.value = true;
          return;
        }
      },
      showUser,
      showGithub,
    };
  },
  name: "Admin",
  components: {
    Users,
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