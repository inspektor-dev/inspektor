import { createRouter, createWebHistory } from 'vue-router'
import Home from '../views/Home.vue'
import Cookie from "js-cookie";

const routes = [
  {
    path: '/',
    name: 'Home',
    component: Home,
    beforeEnter: (_, _1, next) => {
      let cookie = Cookie.get("servertoken")
      if (cookie != null) {
        localStorage.setItem('access-token', cookie)
      }
      let expired = checkTokenExpiration();
      if (!expired) {
        next({ path: "/dashboard" })
        return
      }
      next()
    }
  },
  {
    path: '/dashboard',
    name: 'Dashboard',
    // route level code-splitting
    // this generates a separate chunk (about.[hash].js) for this route
    // which is lazy-loaded when the route is visited.
    component: () => import(/* webpackChunkName: "about" */ '../views/dashboard.vue'),
    beforeEnter: (_, _1, next) => {
      let expired = checkTokenExpiration();
      if (expired){
        next({path: "/"})
        return
      }
      next()
    }
  }
]

const checkTokenExpiration = () => {
  let token = localStorage.getItem('access-token')
  if (token == null){
    return true
  }
  let jwtPayload = JSON.parse(window.atob(token.split('.')[1]))
  let currentEpoch = Math.round(new Date().getTime()/1000)
  let expired = currentEpoch > jwtPayload.exp
  if (expired) {
    localStorage.clear()
  }
  return expired
}

const router = createRouter({
  history: createWebHistory(process.env.BASE_URL),
  routes
})

router.beforeEach((to, _, next) => {
  let cookie = Cookie.get("servertoken")
  if (cookie != null) {
    localStorage.setItem('access-token', cookie)
  }
  let token = localStorage.getItem('access-token')
  if (token == null && to.path != '/') {
    next({ path: '/' })
    return
  }
  next()
})

export default router
