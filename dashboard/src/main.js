import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import naive from 'naive-ui'
import store from './store/store'

createApp(App).
    use(router).
    use(naive).
    use(store).
    mount('#app')
    