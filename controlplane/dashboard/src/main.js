import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import naive from 'naive-ui'
import store from './store/store'

let app = createApp(App)
    app.use(router).
    use(store).
    use(naive).
    mount('#app')
    